use crate::DnsError;
use crate::dns::DnsConfig;
use crate::ns::persistence::{PersistedRecord, PersistedZone, ZonePersistence};
use hickory_server::net::runtime::TokioRuntimeProvider;
use hickory_server::net::xfer::Protocol;
use hickory_server::proto::op::{Header, HeaderCounts, Message, Metadata, ResponseCode};
use hickory_server::proto::rr::{
    LowerName, Name, RData, Record, RecordType,
    rdata::{NS, SOA},
};
use hickory_server::proto::serialize::binary::BinEncodable;
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_server::store::in_memory::InMemoryZoneHandler;
use hickory_server::zone_handler::{
    AxfrPolicy, LookupError, LookupOptions, MessageResponseBuilder, ZoneHandler, ZoneType,
};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// SOA serial floor for today in UTC, formatted as `YYYYMMDD00`.
///
/// The serial grows monotonically within a day; the two trailing digits are a
/// zero-padded per-day revision counter, matching the conventional
/// date-stamped SOA serial scheme.
#[must_use]
pub fn today_serial_base() -> u32 {
    use chrono::Datelike;
    let now = chrono::Utc::now();
    (now.year() as u32) * 1_000_000 + now.month() * 10_000 + now.day() * 100
}

/// Provides authoritative DNS zone management using hickory-server's in-memory zones.
pub struct AuthorityStore {
    config: DnsConfig,
    pub zones: RwLock<HashMap<String, Arc<InMemoryZoneHandler<TokioRuntimeProvider>>>>,
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
        let serial = today_serial_base();
        self.create_zone_with_serial(name, serial).await
    }

    /// Create a new DNS zone, seeding its SOA record with the given serial.
    async fn create_zone_with_serial(&self, name: impl Into<String>, serial: u32) -> bool {
        let zone_name = Self::normalize_zone_name(&name.into());
        let Ok(origin) = Name::from_utf8(&zone_name) else {
            return false;
        };

        let Some(soa_record) = self.build_soa_record(&origin, serial) else {
            return false;
        };

        let handler = Arc::new(InMemoryZoneHandler::empty(
            origin.clone(),
            ZoneType::Primary,
            AxfrPolicy::Deny,
        ));

        if !handler.upsert(soa_record, serial).await {
            return false;
        }

        // Advertise the configured nameservers at the zone apex so that
        // authoritative NS queries resolve to Hackflare's own servers.
        for ns_name in &self.config.nameservers {
            let Ok(ns_name) = Name::from_utf8(ns_name.trim()) else {
                eprintln!("[hackflare:dns] skipping invalid nameserver in config");
                continue;
            };
            handler
                .upsert(
                    Record::from_rdata(origin.clone(), self.config.soa_ttl, RData::NS(NS(ns_name))),
                    serial,
                )
                .await;
        }

        let zone_key = zone_name.trim_end_matches('.').to_string();
        self.zones.write().await.insert(zone_key.clone(), handler);

        if let Some(persistence) = &self.persistence {
            let persisted = PersistedZone {
                name: zone_key.clone(),
                serial,
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
        self.add_record_inner(zone_name, name, rtype, ttl, data, true)
            .await
    }

    /// Add a DNS record to a zone, optionally bumping the SOA serial on success.
    async fn add_record_inner(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
        ttl: u32,
        data: &str,
        bump: bool,
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

        let serial = handler.serial().await;
        let ok = handler
            .upsert(Record::from_rdata(record_name, ttl, rdata), serial)
            .await;

        if ok && bump {
            self.bump_soa_serial(&zone_key).await;
        }

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
        let record_name = match Name::from_utf8(&fqdn) {
            Ok(n) => n,
            Err(_) => return false,
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };

        // Apex NS records are managed by the authoritative server; refuse deletion
        if record_type == RecordType::NS && fqdn == normalized_zone {
            return false;
        }

        let removed = {
            let mut records = handler.records_mut().await;
            let before_count = records.len();
            let target_name = LowerName::new(&record_name);
            records
                .retain(|key, _| !(key.name() == &target_name && key.record_type == record_type));
            before_count != records.len()
        };

        if removed {
            self.bump_soa_serial(&zone_key).await;
            if let Some(persistence) = &self.persistence
                && let Err(e) = persistence
                    .delete_record(zone_name, name, rtype.trim())
                    .await
            {
                eprintln!(
                    "[hackflare:dns] failed to remove record {name} ({rtype}) from storage: {e}"
                );
            }
        }

        removed
    }

    /// Remove a single DNS record value from a zone.
    pub async fn remove_record_value(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
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

        // Apex NS records are managed by the authoritative server; refuse deletion.
        if record_type == RecordType::NS && fqdn == normalized_zone {
            return false;
        }

        let target = Record::from_rdata(record_name.clone(), 0, rdata);
        let target_name = LowerName::new(&record_name);
        let serial = handler.serial().await;

        let removed = {
            let mut records = handler.records_mut().await;
            let key = records
                .keys()
                .find(|k| k.name() == &target_name && k.record_type == record_type)
                .cloned();

            let Some(key) = key else {
                return false;
            };

            let Some(current) = records.get(&key).cloned() else {
                return false;
            };

            // Clone the RecordSet out of the Arc so we can drop a single value.
            let mut new_set = Arc::try_unwrap(current).unwrap_or_else(|arc| (*arc).clone());
            if !new_set.remove(&target, serial) {
                return false;
            }

            if new_set.is_empty() {
                records.remove(&key).is_some()
            } else {
                records.insert(key, Arc::new(new_set));
                true
            }
        };

        if removed {
            self.bump_soa_serial(&zone_key).await;
            if let Some(persistence) = &self.persistence
                && let Err(e) = persistence
                    .delete_record_value(zone_name, name, rtype.trim(), data)
                    .await
            {
                eprintln!(
                    "[hackflare:dns] failed to remove record {name} ({rtype}) value from storage: {e}"
                );
            }
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

    /// Return the zone name that best matches the given name, if any.
    pub async fn find_zone_name(&self, name: &LowerName) -> Option<String> {
        let zones = self.zones.read().await;
        let mut current = name.clone();
        loop {
            let key = current.to_utf8().trim_end_matches('.').to_string();
            if zones.contains_key(&key) {
                return Some(key);
            }
            if current.is_root() {
                return None;
            }
            current = current.base_name();
        }
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
            self.create_zone_with_serial(&zone.name, zone.serial).await;
            for record in zone.records {
                let _ = self
                    .add_record_inner(
                        &zone.name,
                        &record.name,
                        &record.rtype,
                        record.ttl,
                        &record.data,
                        false,
                    )
                    .await;
            }
        }

        Ok(())
    }

    // === Helper Methods ===

    /// Increment a zone's SOA serial and persist the new value.
    ///
    /// The new serial is stamped with the current UTC date and is always
    /// greater than the previous serial: `max(current + 1, today_serial_base())`.
    /// This keeps secondaries in sync while avoiding serial collapse on a new day.
    async fn bump_soa_serial(&self, zone_key: &str) -> Option<u32> {
        let handler = self.zones.read().await.get(zone_key)?.clone();
        let current = handler.serial().await;
        let new_serial = current.saturating_add(1).max(today_serial_base());

        let origin: Name = handler.origin().clone().into();
        // Replace the SOA record; hickory accepts the update because the new
        // serial is strictly greater than the existing one.
        let soa_record = self.build_soa_record(&origin, new_serial)?;
        if !handler.upsert(soa_record, new_serial).await {
            return None;
        }

        if let Some(persistence) = &self.persistence
            && let Err(e) = persistence.update_zone_serial(zone_key, new_serial).await
        {
            eprintln!(
                "[hackflare:dns] failed to persist SOA serial {new_serial} for zone {zone_key}: {e}"
            );
        }

        Some(new_serial)
    }

    /// Normalize zone name (lowercase, trailing dot).
    pub fn normalize_zone_name(name: &str) -> String {
        let trimmed = name.trim().trim_end_matches('.').to_ascii_lowercase();
        if trimmed.is_empty() {
            ".".to_string()
        } else {
            format!("{trimmed}.")
        }
    }

    /// Build fully-qualified domain name from zone and relative name.
    pub fn build_fqdn(zone: &str, name: &str) -> String {
        let normalized = name.trim().trim_end_matches('.').to_ascii_lowercase();

        if normalized.is_empty() || normalized == "@" {
            zone.to_string()
        } else if normalized.ends_with(zone.trim_end_matches('.')) {
            format!("{normalized}.")
        } else {
            format!("{normalized}.{zone}")
        }
    }

    // Build a default SOA record for a zone with the given serial.
    #[allow(clippy::cast_possible_wrap)]
    fn build_soa_record(&self, origin: &Name, serial: u32) -> Option<Record> {
        let mname = Name::from_utf8(&self.config.soa_mname).ok()?;
        let rname = Name::from_utf8(&self.config.soa_rname).ok()?;

        let soa = SOA::new(
            mname,
            rname,
            serial,
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
pub fn find_zone(
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
pub async fn build_auth_response(
    result: Option<Result<hickory_server::zone_handler::AuthLookup, LookupError>>,
    _handler: &InMemoryZoneHandler<TokioRuntimeProvider>,
    request_meta: &Metadata,
    query: &hickory_server::proto::op::LowerQuery,
    recursion_available: bool,
) -> Message {
    let mut response_meta = Metadata::response_from_request(request_meta);
    response_meta.authoritative = true;
    // Advertise recursion when the server can resolve non-hosted queries
    response_meta.recursion_available = recursion_available;

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

        let mut message = build_auth_response(
            result,
            &handler,
            &request.metadata,
            request_info.query,
            self.config.recursion_enabled,
        )
        .await;

        // Clamp UDP response size to prevent amplification attacks
        if request.protocol() == Protocol::Udp {
            let wire_len = message.to_bytes().ok().map(|v| v.len());
            if let Some(len) = wire_len
                && len > self.config.max_edns_payload_size as usize
            {
                message.metadata.truncation = true;
                message.answers.clear();
                message.additionals.clear();
            }
        }

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
