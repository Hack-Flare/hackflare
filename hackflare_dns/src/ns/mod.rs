pub mod authority;
mod hickory;

pub mod config;
pub mod persistence;
pub mod server;

pub use authority::AuthorityStore;
pub use config::NsConfig;
pub use hickory::run_with_hickory;
pub use persistence::{
    MemoryPersistence, PersistedRecord, PersistedZone, PostgresPersistence, ZonePersistence,
};
pub use server::Nameserver;
