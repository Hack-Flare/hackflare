mod dns;
pub mod ns;

pub use dns::config::DnsConfig;
pub use dns::recursive::ensure_root_hints_in_db;
pub use ns::{Nameserver, NsConfig};
