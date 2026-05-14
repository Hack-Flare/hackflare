use crate::dns::DnsConfig;
use crate::ns::persistence::{ZonePersistence, PersistedZone};
use hickory_server::net::runtime::TokioRuntimeProvider;
use hickory_server::proto::rr::{
    rdata::SOA, LowerName, Name, RData, Record, RecordType,
};
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_server::store::in_memory::InMemoryZoneHandler;
use hickory_server::zone_handler::{AxfrPolicy, Catalog, ZoneHandler, ZoneType};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

// Provides authoritative DNS zone management using hickory-server's in-memory zones.
pub struct AuthorityStore {
    config: DnsConfig,
    catalog: RwLock<Catalog>,
    zones: RwLock<HashMap<String, Arc<InMemoryZoneHandler<TokioRuntimeProvider>>>>,
    #[allow(dead_code)]
    persistence: Option<Arc<dyn ZonePersistence>>,
}

impl AuthorityStore {
    pub fn new(config: DnsConfig) -> Self {
        Self {
            config,
            catalog: RwLock::new(Catalog::new()),
            zones: RwLock::new(HashMap::new()),
            persistence: None,
        }
    }

    // Create AuthorityStore with persistence enabled
    #[allow(dead_code)]
    pub fn with_persistence(
        config: DnsConfig,
        persistence: Arc<dyn ZonePersistence>,
    ) -> Self {
        Self {
            config,
            catalog: RwLock::new(Catalog::new()),
            zones: RwLock::new(HashMap::new()),
            persistence: Some(persistence),
        }
    }

    // Create a new DNS zone with default SOA record.
    pub async fn create_zone(&self, name: impl Into<String>) -> bool {
        let zone_name = Self::normalize_zone_name(&name.into());
        let origin = match Name::from_utf8(&zone_name) {
            Ok(n) => n,
            Err(_) => return false,
        };

        let soa_record = match self.build_soa_record(&origin) {
            Some(record) => record,
            None => return false,
        };

        let handler = Arc::new(InMemoryZoneHandler::empty(
            origin.clone(),
            ZoneType::Primary,
            AxfrPolicy::Deny,
        ));

        let serial = Self::parse_u32(&self.config.soa_serial, 1);
        if !handler.upsert(soa_record, serial).await {
            return false;
        }

        let zone_key = zone_name.trim_end_matches('.').to_string();
        self.zones
            .write()
            .await
            .insert(zone_key.clone(), Arc::clone(&handler));

        let zone_name_lower = LowerName::new(&origin);
        let zone_handler: Arc<dyn ZoneHandler> = handler;
        self.catalog
            .write()
            .await
            .upsert(zone_name_lower, vec![zone_handler]);

        true
    }

    // Delete an existing DNS zone.
    pub async fn delete_zone(&self, name: &str) -> bool {
        let zone_name = Self::normalize_zone_name(name);
        let origin = match Name::from_utf8(&zone_name) {
            Ok(n) => n,
            Err(_) => return false,
        };

        let zone_key = zone_name.trim_end_matches('.').to_string();
        let removed = self.zones.write().await.remove(&zone_key).is_some();

        if removed {
            let _ = self
                .catalog
                .write()
                .await
                .remove(&LowerName::new(&origin));
        }
        removed
    }

    // Add a DNS record to a zone.
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

        let handler = match self.zones.read().await.get(&zone_key).cloned() {
            Some(h) => h,
            None => return false,
        };

        let fqdn = Self::build_fqdn(&normalized_zone, name);
        let record_name = match Name::from_utf8(&fqdn) {
            Ok(n) => n,
            Err(_) => return false,
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };
        let Ok(rdata) = RData::try_from_str(record_type, data) else {
            return false;
        };

        let serial = Self::parse_u32(&self.config.soa_serial, 1);
        handler
            .upsert(Record::from_rdata(record_name, ttl, rdata), serial)
            .await
    }

    // Remove a DNS record from a zone.
    pub async fn remove_record(&self, zone_name: &str, name: &str, rtype: &str) -> bool {
        let normalized_zone = Self::normalize_zone_name(zone_name);
        let zone_key = normalized_zone.trim_end_matches('.').to_string();

        let handler = match self.zones.read().await.get(&zone_key).cloned() {
            Some(h) => h,
            None => return false,
        };

        let fqdn = Self::build_fqdn(&normalized_zone, name);
        let record_name = match Name::from_utf8(&fqdn) {
            Ok(n) => n,
            Err(_) => return false,
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };

        let mut records = handler.records_mut().await;
        let before_count = records.len();
        let target_name = LowerName::new(&record_name);
        records.retain(|key, _| {
            !(key.name() == &target_name && key.record_type == record_type)
        });

        before_count != records.len()
    }

    // List all zones.
    pub async fn list_zones(&self) -> Vec<String> {
        self.zones.read().await.keys().cloned().collect()
    }

    // Check if a zone exists for the given name.
    pub async fn contains_zone_for(&self, name: &LowerName) -> bool {
        self.catalog.read().await.find(name).is_some()
    }

    // Handle incoming DNS request using the catalog.
    pub async fn handle_request<R: ResponseHandler, T: hickory_server::net::runtime::Time>(
        &self,
        request: &Request,
        response_handle: R,
    ) -> ResponseInfo {
        let catalog = self.catalog.read().await;
        RequestHandler::handle_request::<R, T>(&*catalog, request, response_handle).await
    }

    // Load all zones from persistence storage
    #[allow(dead_code)]
    pub async fn load_zones_from_storage(&self) -> Result<(), String> {
        let persistence = match &self.persistence {
            Some(p) => p,
            None => return Err("No persistence backend configured".to_string()),
        };

        let zones = persistence
            .load_zones()
            .await
            .map_err(|e| format!("Failed to load zones: {}", e))?;

        for zone in zones {
            self.create_zone(&zone.name).await;

            for record in zone.records {
                let _ = self.add_record(
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

    // Save a zone to persistence storage
    #[allow(dead_code)]
    pub async fn save_zone_to_storage(&self, zone_name: &str) -> Result<(), String> {
        let persistence = match &self.persistence {
            Some(p) => p,
            None => return Err("No persistence backend configured".to_string()),
        };

        let zone_key = Self::normalize_zone_name(zone_name)
            .trim_end_matches('.')
            .to_string();

        // For now, just save zone existence. Full record export would require
        // deeper integration with hickory-server's RecordSet API
        let zone = PersistedZone {
            name: zone_key.clone(),
            records: Vec::new(),
        };

        persistence
            .save_zone(&zone)
            .await
            .map_err(|e| format!("Failed to save zone: {}", e))?;

        Ok(())
    }

    // === Helper Methods ===

    // Normalize zone name (lowercase, trailing dot).
    fn normalize_zone_name(name: &str) -> String {
        let trimmed = name.trim().trim_end_matches('.').to_ascii_lowercase();
        if trimmed.is_empty() {
            ".".to_string()
        } else {
            format!("{}.", trimmed)
        }
    }

    // Build fully-qualified domain name from zone and relative name.
    fn build_fqdn(zone: &str, name: &str) -> String {
        let normalized = name.trim().trim_end_matches('.').to_ascii_lowercase();

        if normalized.is_empty() || normalized == "@" {
            zone.to_string()
        } else if normalized.ends_with(zone.trim_end_matches('.')) {
            format!("{}.", normalized)
        } else {
            format!("{}.{}", normalized, zone)
        }
    }

    // Parse a u32 string with a default fallback.
    fn parse_u32(value: &str, default: u32) -> u32 {
        value.trim().parse().unwrap_or(default)
    }

    // Build a default SOA record for a zone.
    fn build_soa_record(&self, origin: &Name) -> Option<Record> {
        let mname = Name::from_utf8(&self.config.soa_mname).ok()?;
        let rname = Name::from_utf8(&self.config.soa_rname).ok()?;

        let soa = SOA::new(
            mname,
            rname,
            Self::parse_u32(&self.config.soa_serial, 1),
            Self::parse_u32(&self.config.soa_refresh, 3600) as i32,
            Self::parse_u32(&self.config.soa_retry, 1800) as i32,
            Self::parse_u32(&self.config.soa_expire, 604800) as i32,
            Self::parse_u32(&self.config.soa_minimum, 86400),
        );

        Some(Record::from_rdata(
            origin.clone(),
            Self::parse_u32(&self.config.soa_ttl, 3600),
            RData::SOA(soa),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dns::DnsConfig;

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
        let result = store.add_record("example.com", "www", "A", 300, "192.168.1.1").await;
        assert!(result);
    }

    #[tokio::test]
    async fn add_record_fails_for_nonexistent_zone() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        let result = store.add_record("nonexistent.com", "www", "A", 300, "192.168.1.1").await;
        assert!(!result);
    }

    #[tokio::test]
    async fn add_record_with_various_types() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        store.create_zone("example.com").await;

        assert!(store.add_record("example.com", "www", "A", 300, "192.168.1.1").await);
        assert!(store.add_record("example.com", "mail", "MX", 300, "10 mail.example.com").await);
        assert!(store.add_record("example.com", "@", "TXT", 300, "v=spf1 ~all").await);
    }

    #[tokio::test]
    async fn remove_record_succeeds() {
        let config = DnsConfig::default_config();
        let store = AuthorityStore::new(config);

        store.create_zone("example.com").await;
        store.add_record("example.com", "www", "A", 300, "192.168.1.1").await;

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
        assert_eq!(AuthorityStore::normalize_zone_name("example.com"), "example.com.");
        assert_eq!(AuthorityStore::normalize_zone_name("example.com."), "example.com.");
    }

    #[test]
    fn normalize_zone_name_lowercases() {
        assert_eq!(AuthorityStore::normalize_zone_name("Example.COM"), "example.com.");
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

    #[test]
    fn parse_u32_uses_default() {
        assert_eq!(AuthorityStore::parse_u32("invalid", 42), 42);
        assert_eq!(AuthorityStore::parse_u32("", 100), 100);
    }

    #[test]
    fn parse_u32_parses_valid_number() {
        assert_eq!(AuthorityStore::parse_u32("3600", 0), 3600);
        assert_eq!(AuthorityStore::parse_u32(" 1800 ", 0), 1800);
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