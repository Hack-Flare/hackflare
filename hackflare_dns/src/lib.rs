pub mod dns;
pub mod ns;

pub use dns::{DnsConfig, DnsEngine, DnsManager, Record, Zone};
pub use ns::{Nameserver, NsConfig};
