pub mod authoritative;
pub mod config;
pub mod packet;
pub mod recursive;
pub mod wire;

pub use authoritative::{DomainValidationError, check_domain_registration};
pub use config::DnsConfig;
