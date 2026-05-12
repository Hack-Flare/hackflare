use crate::dns::engine::DnsEngine;
use crate::dns::DnsConfig;
use crate::ns::authority::AuthorityStore;
use crate::ns::hickory::run_with_hickory;
use crate::ns::NsConfig;
use std::io;
use std::sync::Arc;

pub struct Nameserver {
    pub config: NsConfig,
    authority: Arc<AuthorityStore>,
    engine: Arc<DnsEngine>,
}

impl Nameserver {
    pub fn new(config: NsConfig) -> Self {
        Self::with_dns_config(config, DnsConfig::from_env())
    }

    pub fn with_dns_config(config: NsConfig, dns_config: DnsConfig) -> Self {
        let engine = DnsEngine::new(crate::dns::manager::DnsManager::new(), dns_config.clone());
        Self {
            authority: Arc::new(AuthorityStore::new(dns_config)),
            config,
            engine: Arc::new(engine),
        }
    }

    pub fn create_zone(&self, name: impl Into<String>) {
        let authority = Arc::clone(&self.authority);
        tokio::runtime::Runtime::new()
            .expect("failed to create runtime")
            .block_on(async move {
                let _ = authority.create_zone(name).await;
            });
    }

    pub fn delete_zone(&self, name: &str) -> bool {
        tokio::runtime::Runtime::new()
            .expect("failed to create runtime")
            .block_on(self.authority.delete_zone(name))
    }

    pub fn add_record(&self, zone_name: &str, name: &str, rtype: &str, ttl: u32, data: &str) -> bool {
        tokio::runtime::Runtime::new()
            .expect("failed to create runtime")
            .block_on(self.authority.add_record(zone_name, name, rtype, ttl, data))
    }

    pub fn remove_record(&self, zone_name: &str, name: &str, rtype: &str) -> bool {
        tokio::runtime::Runtime::new()
            .expect("failed to create runtime")
            .block_on(self.authority.remove_record(zone_name, name, rtype))
    }

    pub fn list_zones(&self) -> Vec<String> {
        tokio::runtime::Runtime::new()
            .expect("failed to create runtime")
            .block_on(self.authority.list_zones())
    }

    pub fn run(&self) -> io::Result<()> {
        run_with_hickory(&self.config, Arc::clone(&self.authority), Arc::clone(&self.engine))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dns::DnsConfig;

    #[test]
    fn nameserver_constructors_set_expected_state() {
        let config = NsConfig {
            bind_addr: "127.0.0.1".to_string(),
            port: 5300,
            zone_file: Some("zones.json".to_string()),
            database_url: None,
        };

        let empty = Nameserver::new(config);
        assert_eq!(empty.config.bind_addr, "127.0.0.1");
        assert_eq!(empty.config.port, 5300);
        assert_eq!(empty.config.zone_file.as_deref(), Some("zones.json"));
        assert!(empty.list_zones().is_empty());

        let with_engine = Nameserver::with_dns_config(NsConfig::default(), DnsConfig::default_config());
        assert!(with_engine.list_zones().is_empty());
    }
}
