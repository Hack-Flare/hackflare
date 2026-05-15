mod authority;
mod hickory;

pub mod config;
pub mod persistence;
pub mod server;

pub use config::NsConfig;
pub use persistence::{
    MemoryPersistence, PersistedRecord, PersistedZone, ZonePersistence,
};
pub use server::Nameserver;
