//! Zone persistence backends for storing DNS zones durably.
//!
//! This module provides pluggable persistence implementations for DNS zones, allowing zones and
//! records to be stored durably and restored on server restart.
//!
//! # Implementations
//!
//! - [`MemoryPersistence`]: In-memory storage for testing and development
//!
//! Implement the [`ZonePersistence`] trait to add your own storage backend (e.g., `PostgreSQL`, `Redis`).
//!
use std::collections::HashMap;
use std::error::Error;

/// Zone data structure for persistence
///
/// Represents a DNS zone that can be stored durably across server restarts.
#[derive(Debug, Clone)]
pub struct PersistedZone {
    pub name: String,
    pub records: Vec<PersistedRecord>,
}

/// DNS record structure for persistence
///
/// Represents a single DNS record within a zone.
#[derive(Debug, Clone)]
pub struct PersistedRecord {
    pub name: String,
    pub rtype: String,
    pub ttl: u32,
    pub data: String,
}

/// Abstract trait for zone persistence
///
/// Implement this trait to add custom storage backends (e.g., `MySQL`, `DynamoDB`, `Redis`).
/// All methods are async to support non-blocking I/O operations.
#[async_trait::async_trait]
pub trait ZonePersistence: Send + Sync {
    /// Load all zones from storage
    ///
    /// Returns a vector of all persisted zones.
    async fn load_zones(&self) -> Result<Vec<PersistedZone>, Box<dyn Error>>;

    /// Load a specific zone by name
    ///
    /// Returns `Ok(Some(zone))` if found, `Ok(None)` if not found, or an error.
    async fn load_zone(&self, zone_name: &str) -> Result<Option<PersistedZone>, Box<dyn Error>>;

    /// Save or update a zone
    ///
    /// This is idempotent - calling it multiple times with the same zone is safe.
    async fn save_zone(&self, zone: &PersistedZone) -> Result<(), Box<dyn Error>>;

    /// Delete a zone
    ///
    /// This is idempotent - deleting a non-existent zone is safe.
    async fn delete_zone(&self, zone_name: &str) -> Result<(), Box<dyn Error>>;

    /// Save a record in a zone
    ///
    /// If the record already exists, it is updated (upserted).
    async fn save_record(
        &self,
        zone_name: &str,
        record: &PersistedRecord,
    ) -> Result<(), Box<dyn Error>>;

    /// Delete a record from a zone
    ///
    /// This is idempotent - deleting a non-existent record is safe.
    async fn delete_record(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
    ) -> Result<(), Box<dyn Error>>;
}

/// In-memory implementation for testing or when no database is available
///
/// This backend stores zones in memory using a `HashMap`. It's useful for:
/// - Unit and integration testing
/// - Development environments without a database
/// - Temporary zone caching
///
/// Note: Zones are lost when the server restarts.
///
/// # Example
///
/// ```
/// use hackflare_dns::ns::MemoryPersistence;
///
/// let storage = MemoryPersistence::new();
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// // storage is ready to use immediately
/// # });
/// ```
pub struct MemoryPersistence {
    zones: parking_lot::RwLock<HashMap<String, PersistedZone>>,
}

impl MemoryPersistence {
    /// Create a new in-memory persistence backend
    #[must_use]
    pub fn new() -> Self {
        Self {
            zones: parking_lot::RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryPersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ZonePersistence for MemoryPersistence {
    async fn load_zones(&self) -> Result<Vec<PersistedZone>, Box<dyn Error>> {
        let zones = self.zones.read();
        Ok(zones.values().cloned().collect())
    }

    async fn load_zone(&self, zone_name: &str) -> Result<Option<PersistedZone>, Box<dyn Error>> {
        let zones = self.zones.read();
        Ok(zones.get(zone_name).cloned())
    }

    async fn save_zone(&self, zone: &PersistedZone) -> Result<(), Box<dyn Error>> {
        self.zones.write().insert(zone.name.clone(), zone.clone());
        Ok(())
    }

    async fn delete_zone(&self, zone_name: &str) -> Result<(), Box<dyn Error>> {
        self.zones.write().remove(zone_name);
        Ok(())
    }

    async fn save_record(
        &self,
        zone_name: &str,
        record: &PersistedRecord,
    ) -> Result<(), Box<dyn Error>> {
        let mut zones = self.zones.write();
        let zone = zones
            .entry(zone_name.to_string())
            .or_insert_with(|| PersistedZone {
                name: zone_name.to_string(),
                records: Vec::new(),
            });
        if let Some(existing) = zone
            .records
            .iter_mut()
            .find(|r| r.name == record.name && r.rtype == record.rtype)
        {
            existing.ttl = record.ttl;
            existing.data.clone_from(&record.data);
        } else {
            zone.records.push(record.clone());
        }
        drop(zones);
        Ok(())
    }

    async fn delete_record(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut zones = self.zones.write();
        if let Some(zone) = zones.get_mut(zone_name) {
            zone.records
                .retain(|r| !(r.name == name && r.rtype == rtype));
        }
        drop(zones);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_persistence_saves_and_loads_zones() {
        let storage = MemoryPersistence::new();

        let zone = PersistedZone {
            name: "example.com".to_string(),
            records: vec![PersistedRecord {
                name: "www".to_string(),
                rtype: "A".to_string(),
                ttl: 300,
                data: "192.168.1.1".to_string(),
            }],
        };

        storage.save_zone(&zone).await.unwrap();

        let loaded = storage.load_zone("example.com").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "example.com");
    }

    #[tokio::test]
    async fn memory_persistence_saves_and_loads_records() {
        let storage = MemoryPersistence::new();

        let record = PersistedRecord {
            name: "example.com".to_string(),
            rtype: "A".to_string(),
            ttl: 300,
            data: "192.168.1.1".to_string(),
        };

        storage.save_record("example.com", &record).await.unwrap();

        let zone = storage.load_zone("example.com").await.unwrap();
        assert!(zone.is_some());
        assert_eq!(zone.unwrap().records.len(), 1);
    }

    #[tokio::test]
    async fn memory_persistence_deletes_zones() {
        let storage = MemoryPersistence::new();

        let zone = PersistedZone {
            name: "example.com".to_string(),
            records: vec![],
        };

        storage.save_zone(&zone).await.unwrap();
        storage.delete_zone("example.com").await.unwrap();

        let loaded = storage.load_zone("example.com").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn memory_persistence_deletes_records() {
        let storage = MemoryPersistence::new();

        let zone = PersistedZone {
            name: "example.com".to_string(),
            records: vec![],
        };

        let record = PersistedRecord {
            name: "www".to_string(),
            rtype: "A".to_string(),
            ttl: 300,
            data: "192.168.1.1".to_string(),
        };

        storage.save_zone(&zone).await.unwrap();
        storage.save_record("example.com", &record).await.unwrap();
        storage
            .delete_record("example.com", "www", "A")
            .await
            .unwrap();

        let zone = storage.load_zone("example.com").await.unwrap();
        assert_eq!(zone.unwrap().records.len(), 0);
    }
}
