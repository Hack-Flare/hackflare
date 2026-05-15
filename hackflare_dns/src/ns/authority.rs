use crate::DnsError;
use crate::dns::DnsConfig;
use crate::ns::persistence::{PersistedRecord, PersistedZone, ZonePersistence};
use hickory_server::net::runtime::TokioRuntimeProvider;
use hickory_server::proto::op::{Header, HeaderCounts, Message, Metadata, ResponseCode};
use hickory_server::proto::rr::{LowerName, Name, RData, Record, RecordType, rdata::SOA};
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_server::store::in_memory::InMemoryZoneHandler;
use hickory_server::zone_handler::{
    AxfrPolicy, LookupError, LookupOptions, MessageResponseBuilder, ZoneHandler, ZoneType,
};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Provides authoritative DNS zone management using hickory-server's in-memory zones.
pub struct AuthorityStore {
    config: DnsConfig,
    zones: RwLock<HashMap<String, Arc<InMemoryZoneHandler<TokioRuntimeProvider>>>>,
    persistence: Option<Arc<dyn ZonePersistence>>,
}

impl AuthorityStore {
    pub fn new(config: DnsConfig) -> Self {
        Self {
            config,
            zones: RwLock::new(HashMap::new()),
            persistence: None,
        }
    }

    /// Create `AuthorityStore` with persistence enabled
    pub fn with_persistence(config: DnsConfig, persistence: Arc<dyn ZonePersistence>) -> Self {
        Self {
            config,
            zones: RwLock::new(HashMap::new()),
            persistence: Some(persistence),
        }
    }

    /// Create a new DNS zone with default SOA record.
    pub async fn create_zone(&self, name: impl Into<String>) -> bool {
        let zone_name = Self::normalize_zone_name(&name.into());
        let Ok(origin) = Name::from_utf8(&zone_name) else {
            return false;
        };

        let Some(soa_record) = self.build_soa_record(&origin) else {
            return false;
        };

        let handler = Arc::new(InMemoryZoneHandler::empty(
            origin.clone(),
            ZoneType::Primary,
            AxfrPolicy::Deny,
        ));

        if !handler.upsert(soa_record, self.config.soa_serial).await {
            return false;
        }

        let zone_key = zone_name.trim_end_matches('.').to_string();
        self.zones.write().await.insert(zone_key.clone(), handler);

        if let Some(persistence) = &self.persistence {
            let persisted = PersistedZone {
                name: zone_key.clone(),
                records: Vec::new(),
            };
            if let Err(e) = persistence.save_zone(&persisted).await {
                eprintln!("[hackflare:dns] failed to persist zone {zone_key}: {e}");
            }
        }

        true
    }

    /// Delete an existing DNS zone.
    pub async fn delete_zone(&self, name: &str) -> bool {
        let zone_name = Self::normalize_zone_name(name);
        let zone_key = zone_name.trim_end_matches('.').to_string();
        let removed = self.zones.write().await.remove(&zone_key).is_some();

        if removed
            && let Some(persistence) = &self.persistence
            && let Err(e) = persistence.delete_zone(&zone_key).await
        {
            eprintln!("[hackflare:dns] failed to delete zone {zone_key} from storage: {e}");
        }
        removed
    }

    /// Add a DNS record to a zone.
    pub async fn add_record(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
        ttl: u32,
        data: &str,
    ) -> bool {
        let normalized_zone = Self::normalize_zone_name(zone_name);
        let zone_key = normalized_zone.trim_end_matches('.').to_string();

        let Some(handler) = self.zones.read().await.get(&zone_key).cloned() else {
            return false;
        };

        let fqdn = Self::build_fqdn(&normalized_zone, name);
        let Ok(record_name) = Name::from_utf8(&fqdn) else {
            return false;
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };
        let Ok(rdata) = RData::try_from_str(record_type, data) else {
            return false;
        };

        let ok = handler
            .upsert(
                Record::from_rdata(record_name, ttl, rdata),
                self.config.soa_serial,
            )
            .await;

        if ok && let Some(persistence) = &self.persistence {
            let record = PersistedRecord {
                name: name.to_string(),
                rtype: rtype.to_string(),
                ttl,
                data: data.to_string(),
            };
            if let Err(e) = persistence.save_record(zone_name, &record).await {
                eprintln!(
                    "[hackflare:dns] failed to persist record {name} ({rtype}) in zone {zone_name}: {e}"
                );
            }
        }

        ok
    }

    /// Remove a DNS record from a zone.
    pub async fn remove_record(&self, zone_name: &str, name: &str, rtype: &str) -> bool {
        let normalized_zone = Self::normalize_zone_name(zone_name);
        let zone_key = normalized_zone.trim_end_matches('.').to_string();

        let Some(handler) = self.zones.read().await.get(&zone_key).cloned() else {
            return false;
        };

        let fqdn = Self::build_fqdn(&normalized_zone, name);
        let Ok(record_name) = Name::from_utf8(&fqdn) else {
            return false;
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };

        let mut records = handler.records_mut().await;
        let before_count = records.len();
        let target_name = LowerName::new(&record_name);
        records.retain(|key, _| !(key.name() == &target_name && key.record_type == record_type));

        let removed = before_count != records.len();

        if removed
            && let Some(persistence) = &self.persistence
            && let Err(e) = persistence
                .delete_record(zone_name, name, rtype.trim())
                .await
        {
            eprintln!("[hackflare:dns] failed to remove record {name} ({rtype}) from storage: {e}");
        }

        removed
    }

    /// List all zones.
    pub async fn list_zones(&self) -> Vec<String> {
        self.zones.read().await.keys().cloned().collect()
    }

    /// Check if a zone exists for the given name.
    pub async fn contains_zone_for(&self, name: &LowerName) -> bool {
        let zones = self.zones.read().await;
        find_zone(&zones, name).is_some()
    }

    /// Load all zones from persistence storage.
    pub async fn load_zones_from_storage(&self) -> Result<(), DnsError> {
        let Some(persistence) = &self.persistence else {
            return Err(DnsError::PersistenceUnconfigured);
        };

        let zones = persistence
            .load_zones()
            .await
            .map_err(|e| DnsError::PersistenceOperation(format!("{e}")))?;

        for zone in zones {
            self.create_zone(&zone.name).await;
            for record in zone.records {
                let _ = self
                    .add_record(
                        &zone.name,
                        &record.name,
                        &record.rtype,
                        record.ttl,
                        &record.data,
                    )
                    .await;
            }
        }

        Ok(())
    }

    // === Helper Methods ===

    /// Normalize zone name (lowercase, trailing dot).
    fn normalize_zone_name(name: &str) -> String {
        let trimmed = name.trim().trim_end_matches('.').to_ascii_lowercase();
        if trimmed.is_empty() {
            ".".to_string()
        } else {
            format!("{trimmed}.")
        }
    }

    /// Build fully-qualified domain name from zone and relative name.
    fn build_fqdn(zone: &str, name: &str) -> String {
        let normalized = name.trim().trim_end_matches('.').to_ascii_lowercase();

        if normalized.is_empty() || normalized == "@" {
            zone.to_string()
        } else if normalized.ends_with(zone.trim_end_matches('.')) {
            format!("{normalized}.")
        } else {
            format!("{normalized}.{zone}")
        }
    }

    // Build a default SOA record for a zone.
    #[allow(clippy::cast_possible_wrap)]
    fn build_soa_record(&self, origin: &Name) -> Option<Record> {
        let mname = Name::from_utf8(&self.config.soa_mname).ok()?;
        let rname = Name::from_utf8(&self.config.soa_rname).ok()?;

        let soa = SOA::new(
            mname,
            rname,
            self.config.soa_serial,
            self.config.soa_refresh as i32,
            self.config.soa_retry as i32,
            self.config.soa_expire as i32,
            self.config.soa_minimum,
        );

        Some(Record::from_rdata(
            origin.clone(),
            self.config.soa_ttl,
            RData::SOA(soa),
        ))
    }
}

// ── Zone lookup & query handling ──

/// Find the best-matching zone handler by walking parent suffixes.
fn find_zone(
    zones: &HashMap<String, Arc<InMemoryZoneHandler<TokioRuntimeProvider>>>,
    name: &LowerName,
) -> Option<Arc<InMemoryZoneHandler<TokioRuntimeProvider>>> {
    let mut current = name.clone();
    loop {
        let key = current.to_utf8().trim_end_matches('.').to_string();
        if let Some(handler) = zones.get(&key) {
            return Some(handler.clone());
        }
        if current.is_root() {
            return None;
        }
        current = current.base_name();
    }
}

/// Build a response message for an authoritative zone lookup.
async fn build_auth_response(
    result: Option<Result<hickory_server::zone_handler::AuthLookup, LookupError>>,
    _handler: &InMemoryZoneHandler<TokioRuntimeProvider>,
    request_meta: &Metadata,
    query: &hickory_server::proto::op::LowerQuery,
) -> Message {
    let mut response_meta = Metadata::response_from_request(request_meta);
    response_meta.authoritative = true;

    let mut message = Message::new(
        response_meta.id,
        response_meta.message_type,
        response_meta.op_code,
    );
    message.add_query(query.original().clone());

    let Some(result) = result else {
        response_meta.response_code = ResponseCode::ServFail;
        message.metadata = response_meta;
        return message;
    };

    match result {
        Ok(mut auth_lookup) => {
            response_meta.response_code = ResponseCode::NoError;
            if let Some(adds) = auth_lookup.take_additionals() {
                message.additionals.extend(adds.iter().cloned());
            }
            let is_referral = auth_lookup.iter().next().is_some_and(|r| {
                r.record_type() == RecordType::NS
                    && query.query_type() != RecordType::NS
                    && query.query_type() != RecordType::ANY
            });
            if is_referral {
                message.authorities.extend(auth_lookup.iter().cloned());
            } else {
                message.answers.extend(auth_lookup.iter().cloned());
            }
            message.metadata = response_meta;
            message
        }
        Err(e) => {
            if e.is_nx_domain() {
                response_meta.response_code = ResponseCode::NXDomain;
            } else if matches!(&e, LookupError::ResponseCode(rc) if *rc == ResponseCode::Refused || *rc == ResponseCode::NotAuth)
            {
                response_meta.response_code = *match &e {
                    LookupError::ResponseCode(rc) => rc,
                    _ => &ResponseCode::ServFail,
                };
            } else {
                response_meta.response_code = ResponseCode::NoError;
            }
            message.metadata = response_meta;
            message
        }
    }
}

/// Send a simple error response.
async fn send_error_response<R: ResponseHandler>(
    request: &Request,
    code: ResponseCode,
    mut response_handle: R,
) -> ResponseInfo {
    let mut metadata = Metadata::response_from_request(&request.metadata);
    metadata.response_code = code;
    let response = MessageResponseBuilder::from_message_request(request).build_no_records(metadata);
    response_handle
        .send_response(response)
        .await
        .unwrap_or_else(|_| {
            ResponseInfo::from(Header {
                metadata,
                counts: HeaderCounts::default(),
            })
        })
}

#[async_trait::async_trait]
impl RequestHandler for AuthorityStore {
    async fn handle_request<R: ResponseHandler, T: hickory_server::net::runtime::Time>(
        &self,
        request: &Request,
        response_handle: R,
    ) -> ResponseInfo {
        let mut response_handle = response_handle;
        let Ok(request_info) = request.request_info() else {
            return send_error_response(request, ResponseCode::FormErr, response_handle).await;
        };

        let query_name = request_info.query.name().clone();

        let handler = {
            let zones = self.zones.read().await;
            find_zone(&zones, &query_name)
        };

        let Some(handler) = handler else {
            return send_error_response(request, ResponseCode::Refused, response_handle).await;
        };

        let lookup_options = LookupOptions::from_edns(request.edns.as_ref());
        let (cf, _) = handler.search(request, lookup_options).await;
        let result = cf.map_result();

        let message =
            build_auth_response(result, &handler, &request.metadata, request_info.query).await;

        let mut builder = MessageResponseBuilder::from_message_request(request);
        if let Some(edns) = &request.edns {
            builder.edns(edns);
        }
        let response = builder.build(
            message.metadata,
            &message.answers,
            &message.authorities,
            std::iter::empty(),
            &message.additionals,
        );

        response_handle
            .send_response(response)
            .await
            .unwrap_or_else(|_| {
                ResponseInfo::from(Header {
                    metadata: Metadata::response_from_request(&request.metadata),
                    counts: HeaderCounts::default(),
                })
            })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_zone_succeeds() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        let result = store.create_zone("example.com").await;
        assert!(result);

        let zones = store.list_zones().await;
        assert!(zones.contains(&"example.com".to_string()));
    }

    #[tokio::test]
    async fn create_zone_normalizes_names() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        let result = store.create_zone("Example.COM.").await;
        assert!(result);

        let zones = store.list_zones().await;
        assert!(zones.contains(&"example.com".to_string()));
    }

    #[tokio::test]
    async fn delete_zone_succeeds() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        store.create_zone("example.com").await;
        let result = store.delete_zone("example.com").await;
        assert!(result);

        let zones = store.list_zones().await;
        assert!(!zones.contains(&"example.com".to_string()));
    }

    #[tokio::test]
    async fn add_record_creates_valid_a_record() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        store.create_zone("example.com").await;
        let result = store
            .add_record("example.com", "www", "A", 300, "192.168.1.1")
            .await;
        assert!(result);
    }

    #[tokio::test]
    async fn add_record_fails_for_nonexistent_zone() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        let result = store
            .add_record("nonexistent.com", "www", "A", 300, "192.168.1.1")
            .await;
        assert!(!result);
    }

    #[tokio::test]
    async fn add_record_with_various_types() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        store.create_zone("example.com").await;

        assert!(
            store
                .add_record("example.com", "www", "A", 300, "192.168.1.1")
                .await
        );
        assert!(
            store
                .add_record("example.com", "mail", "MX", 300, "10 mail.example.com")
                .await
        );
        assert!(
            store
                .add_record("example.com", "@", "TXT", 300, "v=spf1 ~all")
                .await
        );
    }

    #[tokio::test]
    async fn remove_record_succeeds() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        store.create_zone("example.com").await;
        store
            .add_record("example.com", "www", "A", 300, "192.168.1.1")
            .await;

        let result = store.remove_record("example.com", "www", "A").await;
        assert!(result);
    }

    #[tokio::test]
    async fn remove_record_fails_for_nonexistent() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        store.create_zone("example.com").await;

        let result = store.remove_record("example.com", "nonexistent", "A").await;
        assert!(!result);
    }

    #[test]
    fn normalize_zone_name_adds_trailing_dot() {
        assert_eq!(
            AuthorityStore::normalize_zone_name("example.com"),
            "example.com."
        );
        assert_eq!(
            AuthorityStore::normalize_zone_name("example.com."),
            "example.com."
        );
    }

    #[test]
    fn normalize_zone_name_lowercases() {
        assert_eq!(
            AuthorityStore::normalize_zone_name("Example.COM"),
            "example.com."
        );
    }

    #[test]
    fn build_fqdn_handles_apex() {
        assert_eq!(
            AuthorityStore::build_fqdn("example.com.", "@"),
            "example.com."
        );
        assert_eq!(
            AuthorityStore::build_fqdn("example.com.", ""),
            "example.com."
        );
    }

    #[test]
    fn build_fqdn_constructs_subdomain() {
        assert_eq!(
            AuthorityStore::build_fqdn("example.com.", "www"),
            "www.example.com."
        );
    }

    #[test]
    fn build_fqdn_handles_fully_qualified() {
        assert_eq!(
            AuthorityStore::build_fqdn("example.com.", "www.example.com"),
            "www.example.com."
        );
    }

    #[tokio::test]
    async fn contains_zone_for_returns_true_for_existing_zone() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        store.create_zone("example.com").await;

        let name = LowerName::new(&Name::from_utf8("example.com.").unwrap());
        assert!(store.contains_zone_for(&name).await);
    }

    #[tokio::test]
    async fn contains_zone_for_returns_false_for_missing_zone() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        let name = LowerName::new(&Name::from_utf8("nonexistent.com.").unwrap());
        assert!(!store.contains_zone_for(&name).await);
    }
}
