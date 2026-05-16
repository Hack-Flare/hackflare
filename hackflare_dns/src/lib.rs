//! # Hackflare DNS
//!
//! A high-performance, authoritative and recursive DNS server crate built on top of hickory-server.
//!
//! ## Features
//!
//! - **Authoritative DNS Zones**: Host your own DNS zones with full control
//! - **Recursive Resolution**: Resolve external domains through DNS root servers
//! - **Zone Persistence**: Pluggable backends via [`ZonePersistence`] trait
//! - **In-Memory Storage**: Fast zone operations with [`MemoryPersistence`] for testing
//! - **Metrics Collection**: Track DNS queries (UDP/TCP) counters
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
//! ## Persistence
//!
//! The crate provides a pluggable persistence interface through the [`ZonePersistence`] trait.
//! Implement it to store zones and records in any backend (e.g., `PostgreSQL`, `Redis`, filesystem).
//!
//! - **[`MemoryPersistence`]**: In-memory storage for testing and development
//! - **[`PostgresPersistence`]**: PostgreSQL-backed storage for production (consumer-managed pool)
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
//! - [`ns::MemoryPersistence`] - In-memory persistence backend
//! - [`ns::PostgresPersistence`] - PostgreSQL persistence backend (consumer-managed pool)
//! - [`Nameserver`] - Main API for zone management and DNS serving
//!
//! [`ZonePersistence`]: ns::ZonePersistence
//! [`MemoryPersistence`]: ns::MemoryPersistence
//! [`PostgresPersistence`]: ns::PostgresPersistence

mod dns;
mod error;
pub mod ns;

pub use dns::config::DnsConfig;
pub use error::DnsError;
pub use ns::{Nameserver, NsConfig};
