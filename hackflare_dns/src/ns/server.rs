use crate::dns::DnsConfig;
use crate::error::DnsError;
use crate::ns::authority::AuthorityStore;
use crate::ns::hickory::run_with_hickory;
use crate::ns::persistence::ZonePersistence;
use crate::NsConfig;
use std::io;
use std::sync::Arc;

pub struct Nameserver {
    pub config: NsConfig,
    dns_config: DnsConfig,
    authority: Arc<AuthorityStore>,
    runtime: tokio::runtime::Runtime,
}

impl Nameserver {
    /// Create a new nameserver with default DNS configuration
    ///
    /// Configuration is loaded from environment variables with sensible defaults.
    /// Zones are stored in-memory only and will be lost on restart.
    ///
    /// To enable persistence, use [`with_persistence`](Self::with_persistence) instead.
    ///
    /// # Errors
    ///
    /// Returns an error if the Tokio runtime cannot be created.
    pub fn new(config: NsConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_dns_config(config, DnsConfig::from_env())
    }

    /// Create a nameserver with custom DNS configuration
    ///
    /// Allows you to override DNS engine settings while using in-memory zone storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the Tokio runtime cannot be created.
    pub fn with_dns_config(config: NsConfig, dns_config: DnsConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            authority: Arc::new(AuthorityStore::new(dns_config.clone())),
            config,
            dns_config,
            runtime: tokio::runtime::Runtime::new()?,
        })
    }

    /// Create a nameserver with persistent zone storage
    ///
    /// Zones can be loaded from and saved to the persistence backend.
    /// Call [`load_zones_from_storage`](AuthorityStore::load_zones_from_storage)
    /// on startup to restore persisted zones.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hackflare_dns::{Nameserver, NsConfig, DnsConfig};
    /// use hackflare_dns::ns::PostgresPersistence;
    /// use std::sync::Arc;
    ///
    /// let persistence = Arc::new(PostgresPersistence::new(
    ///     "postgresql://user:pass@localhost/dns"
    /// ));
    /// persistence.init_schema()?;
    ///
    /// let nameserver = Nameserver::with_persistence(
    ///     NsConfig::default(),
    ///     DnsConfig::from_env(),
    ///     persistence
    /// )?;
    ///
    /// // Restore zones on startup
    /// nameserver.load_zones_from_storage().map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the Tokio runtime cannot be created.
    pub fn with_persistence(
        config: NsConfig,
        dns_config: DnsConfig,
        persistence: Arc<dyn ZonePersistence>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            authority: Arc::new(AuthorityStore::with_persistence(dns_config.clone(), persistence)),
            config,
            dns_config,
            runtime: tokio::runtime::Runtime::new()?,
        })
    }

    /// Create a new DNS zone
    ///
    /// Creates an authoritative zone with a default SOA record.
    /// If persistence is enabled, call [`AuthorityStore::save_zone_to_storage`]
    /// to persist the zone.
    pub fn create_zone(&self, name: impl Into<String>) {
        let authority = Arc::clone(&self.authority);
        self.runtime.block_on(async move {
            let _ = authority.create_zone(name).await;
        });
    }

    /// Delete an existing DNS zone
    ///
    /// Returns `true` if the zone was deleted, `false` if it didn't exist.
    pub fn delete_zone(&self, name: &str) -> bool {
        self.runtime.block_on(self.authority.delete_zone(name))
    }

    /// Add a DNS record to a zone
    ///
    /// # Arguments
    ///
    /// * `zone_name` - Name of the zone (e.g., "example.com")
    /// * `name` - Record name, relative to zone (e.g., "www", "@" for apex)
    /// * `rtype` - Record type (e.g., "A", "AAAA", "MX", "TXT")
    /// * `ttl` - Time-to-live in seconds
    /// * `data` - Record data (e.g., IP address for A records)
    ///
    /// Returns `true` if the record was added, `false` if the zone doesn't exist.
    pub fn add_record(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
        ttl: u32,
        data: &str,
    ) -> bool {
        self.runtime
            .block_on(self.authority.add_record(zone_name, name, rtype, ttl, data))
    }

    /// Remove a DNS record from a zone
    ///
    /// # Arguments
    ///
    /// * `zone_name` - Name of the zone
    /// * `name` - Record name
    /// * `rtype` - Record type
    ///
    /// Returns `true` if the record was removed, `false` if it didn't exist.
    pub fn remove_record(&self, zone_name: &str, name: &str, rtype: &str) -> bool {
        self.runtime
            .block_on(self.authority.remove_record(zone_name, name, rtype))
    }

    /// Load all zones from persistence storage
    ///
    /// Requires the nameserver to have been created with [`with_persistence`](Self::with_persistence).
    ///
    /// # Errors
    ///
    /// Returns an error if no persistence backend is configured, or if loading from storage fails.
    pub fn load_zones_from_storage(&self) -> Result<(), DnsError> {
        self.runtime
            .block_on(self.authority.load_zones_from_storage())
    }

    /// List all hosted zones
    pub fn list_zones(&self) -> Vec<String> {
        self.runtime.block_on(self.authority.list_zones())
    }

    /// Start the DNS server
    ///
    /// This binds to the address and port specified in the configuration
    /// and begins handling DNS requests. This function blocks indefinitely.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the server fails to start (e.g., port in use).
    pub fn run(&self) -> io::Result<()> {
        run_with_hickory(
            &self.config,
            Arc::clone(&self.authority),
            self.dns_config.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nameserver_constructors_set_expected_state() {
        let config = NsConfig {
            bind_addr: "127.0.0.1".to_string(),
            port: 5300,
            zone_file: Some("zones.json".to_string()),
            database_url: None,
        };

        let empty = Nameserver::new(config).unwrap();
        assert_eq!(empty.config.bind_addr, "127.0.0.1");
        assert_eq!(empty.config.port, 5300);
        assert_eq!(empty.config.zone_file.as_deref(), Some("zones.json"));
        assert!(empty.list_zones().is_empty());

        let with_engine =
            Nameserver::with_dns_config(NsConfig::default(), DnsConfig::default_config()).unwrap();
        assert!(with_engine.list_zones().is_empty());
    }
}
