//! # Hackflare DNS
//!
//! A high-performance, authoritative and recursive DNS server crate built on top of hickory-server.
//!
//! ## Features
//!
//! - **Authoritative DNS Zones**: Host your own DNS zones with full control
//! - **Recursive Resolution**: Resolve external domains through DNS root servers
//! - **Zone Persistence**: Optional `PostgreSQL` backend for durable zone storage
//! - **In-Memory Storage**: Fast zone operations with optional disk/database persistence
//! - **Metrics Collection**: Track DNS queries (UDP/TCP) with `PostgreSQL` logging
//!
//! ## Quick Start
//!
//! ### Without Persistence
//!
//! ```rust,no_run
//! use hackflare_dns::{Nameserver, NsConfig};
//!
//! let config = NsConfig {
//!     bind_addr: "0.0.0.0".to_string(),
//!     port: 53,
//!     zone_file: None,
//!     database_url: None,
//! };
//!
//! let nameserver = Nameserver::new(config)?;
//! nameserver.create_zone("example.com");
//! nameserver.add_record("example.com", "www", "A", 300, "192.0.2.1");
//!
//! // Start the DNS server
//! nameserver.run()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### With `PostgreSQL` Persistence
//!
//! ```rust,no_run
//! use hackflare_dns::{Nameserver, NsConfig, DnsConfig};
//! use hackflare_dns::ns::PostgresPersistence;
//! use std::sync::Arc;
//!
//! // Initialize PostgreSQL persistence
//! let persistence = Arc::new(PostgresPersistence::new(
//!     "postgresql://user:password@localhost/dns_db"
//! ));
//! persistence.init_schema()?;
//!
//! let config = NsConfig {
//!     bind_addr: "0.0.0.0".to_string(),
//!     port: 53,
//!     zone_file: None,
//!     database_url: Some("postgresql://user:password@localhost/dns_db".to_string()),
//! };
//!
//! let nameserver = Nameserver::with_persistence(
//!     config,
//!     DnsConfig::from_env(),
//!     persistence.clone(),
//! )?;
//!
//! // Load zones from database on startup
//! nameserver.load_zones_from_storage().map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
//!
//! nameserver.run()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Persistence
//!
//! The crate provides pluggable persistence backends through the [`ZonePersistence`] trait:
//!
//! - **[`PostgresPersistence`]**: Production-grade `PostgreSQL` backend with automatic schema creation
//! - **[`MemoryPersistence`]**: In-memory storage for testing and development
//!
//! Implement the [`ZonePersistence`] trait to add your own storage backend.
//!
//! ## Configuration
//!
//! Both DNS and nameserver behavior is controlled through configuration:
//!
//! - [`DnsConfig`]: DNS engine settings (recursion, SOA defaults, timeouts)
//! - [`NsConfig`]: Nameserver settings (bind address, port, database URL)
//!
//! Configuration is loaded from environment variables with sensible defaults.
//!
//! ## See Also
//!
//! - [`ns::PostgresPersistence`] - `PostgreSQL` persistence backend
//! - [`ns::MemoryPersistence`] - In-memory persistence backend
//! - [`Nameserver`] - Main API for zone management and DNS serving
//!
//! [`ZonePersistence`]: ns::ZonePersistence
//! [`PostgresPersistence`]: ns::PostgresPersistence
//! [`MemoryPersistence`]: ns::MemoryPersistence

mod dns;
pub mod ns;

pub use dns::config::DnsConfig;
pub use dns::recursive::ensure_root_hints_in_db;
pub use ns::{Nameserver, NsConfig};
