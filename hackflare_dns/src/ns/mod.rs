pub mod authority;

pub mod config;
pub mod handler;
pub mod persistence;
pub mod server;

pub use authority::AuthorityStore;
pub use config::NsConfig;
pub use handler::run;
pub use persistence::{
    MemoryPersistence, PersistedRecord, PersistedZone, PostgresPersistence, ZonePersistence,
};
pub use server::Nameserver;
