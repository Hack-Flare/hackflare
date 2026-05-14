//! Zone persistence backends for storing DNS zones durably.
//!
//! This module provides pluggable persistence implementations for DNS zones, allowing zones and
//! records to be stored durably and restored on server restart.
//!
//! # Implementations
//!
//! - [`PostgresPersistence`]: Production-grade PostgreSQL backend
//! - [`MemoryPersistence`]: In-memory storage for testing
//!
//! # Example: Using PostgreSQL Persistence
//!
//! ```no_run
//! use hackflare_dns::ns::PostgresPersistence;
//! use std::sync::Arc;
//!
//! let persistence = Arc::new(PostgresPersistence::new(
//!     "postgresql://user:password@localhost/dns"
//! ));
//!
//! // Initialize the schema (required once)
//! persistence.init_schema()?;
//!
//! // Load existing zones
//! let zones = persistence.load_zones().await?;
//!
//! // Save a zone
//! let zone = PersistedZone {
//!     name: "example.com".to_string(),
//!     records: vec![],
//! };
//! persistence.save_zone(&zone).await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use postgres::Client;
use postgres::NoTls;
use std::collections::HashMap;
use std::error::Error;

// Zone data structure for persistence
//
// Represents a DNS zone that can be stored durably across server restarts.
#[derive(Debug, Clone)]
pub struct PersistedZone {
    pub name: String,
    pub records: Vec<PersistedRecord>,
}

// DNS record structure for persistence
//
// Represents a single DNS record within a zone.
#[derive(Debug, Clone)]
pub struct PersistedRecord {
    pub name: String,
    pub rtype: String,
    pub ttl: u32,
    pub data: String,
}

// Abstract trait for zone persistence
//
// Implement this trait to add custom storage backends (e.g., MySQL, DynamoDB, Redis).
// All methods are async to support non-blocking I/O operations.
#[async_trait::async_trait]
pub trait ZonePersistence: Send + Sync {
    // Load all zones from storage
    //
    // Returns a vector of all persisted zones.
    async fn load_zones(&self) -> Result<Vec<PersistedZone>, Box<dyn Error>>;

    // Load a specific zone by name
    //
    // Returns `Ok(Some(zone))` if found, `Ok(None)` if not found, or an error.
    async fn load_zone(&self, zone_name: &str) -> Result<Option<PersistedZone>, Box<dyn Error>>;

    // Save or update a zone
    //
    // This is idempotent - calling it multiple times with the same zone is safe.
    async fn save_zone(&self, zone: &PersistedZone) -> Result<(), Box<dyn Error>>;

    // Delete a zone
    //
    // This is idempotent - deleting a non-existent zone is safe.
    async fn delete_zone(&self, zone_name: &str) -> Result<(), Box<dyn Error>>;

    // Save a record in a zone
    //
    // If the record already exists, it is updated (upserted).
    async fn save_record(
        &self,
        zone_name: &str,
        record: &PersistedRecord,
    ) -> Result<(), Box<dyn Error>>;

    // Delete a record from a zone
    //
    // This is idempotent - deleting a non-existent record is safe.
    async fn delete_record(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
    ) -> Result<(), Box<dyn Error>>;
}

// PostgreSQL implementation of zone persistence
//
// This is the recommended persistence backend for production use.
//
// # Example
//
// ```no_run
// use hackflare_dns::ns::PostgresPersistence;
//
// let persistence = PostgresPersistence::new(
//     "postgresql://user:password@localhost/dns_db"
// );
//
// // Initialize schema (run once)
// persistence.init_schema()?;
// # Ok::<(), Box<dyn std::error::Error>>(())
// ```
pub struct PostgresPersistence {
    connection_string: String,
}

impl PostgresPersistence {
    // Create a new PostgreSQL persistence backend
    //
    // # Arguments
    //
    // * `connection_string` - PostgreSQL connection URL (e.g., "postgresql://user:pass@host/db")
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
        }
    }

    // Initialize the database schema
    //
    // This creates the necessary tables (`dns_zones` and `dns_records`) if they don't exist.
    // Call this once during application startup.
    //
    // # Returns
    //
    // Returns an error if the database is unreachable or schema creation fails.
    pub fn init_schema(&self) -> Result<(), Box<dyn Error>> {
        let mut client = Client::connect(&self.connection_string, NoTls)?;

        client.execute(
            "CREATE TABLE IF NOT EXISTS dns_zones (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) UNIQUE NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            &[],
        )?;

        client.execute(
            "CREATE TABLE IF NOT EXISTS dns_records (
                id SERIAL PRIMARY KEY,
                zone_id INT NOT NULL,
                name VARCHAR(255) NOT NULL,
                rtype VARCHAR(10) NOT NULL,
                ttl INT NOT NULL,
                data TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (zone_id) REFERENCES dns_zones(id) ON DELETE CASCADE,
                UNIQUE(zone_id, name, rtype)
            )",
            &[],
        )?;

        client.execute(
            "CREATE INDEX IF NOT EXISTS idx_records_zone_id ON dns_records(zone_id)",
            &[],
        )?;

        client.execute(
            "CREATE INDEX IF NOT EXISTS idx_records_name ON dns_records(name)",
            &[],
        )?;

        Ok(())
    }

    fn get_connection(&self) -> Result<Client, Box<dyn Error>> {
        Ok(Client::connect(&self.connection_string, NoTls)?)
    }
}

#[async_trait::async_trait]
impl ZonePersistence for PostgresPersistence {
    async fn load_zones(&self) -> Result<Vec<PersistedZone>, Box<dyn Error>> {
        let mut client = self.get_connection()?;
        let mut zones = Vec::new();

        let zone_rows = client.query("SELECT id, name FROM dns_zones", &[])?;

        for zone_row in zone_rows {
            let zone_id: i32 = zone_row.get(0);
            let zone_name: String = zone_row.get(1);

            let records = client.query(
                "SELECT name, rtype, ttl, data FROM dns_records WHERE zone_id = $1",
                &[&zone_id],
            )?;

            let mut zone_records = Vec::new();
            for record_row in records {
                zone_records.push(PersistedRecord {
                    name: record_row.get(0),
                    rtype: record_row.get(1),
                    ttl: record_row.get(2),
                    data: record_row.get(3),
                });
            }

            zones.push(PersistedZone {
                name: zone_name,
                records: zone_records,
            });
        }

        Ok(zones)
    }

    async fn load_zone(&self, zone_name: &str) -> Result<Option<PersistedZone>, Box<dyn Error>> {
        let mut client = self.get_connection()?;

        let zone_row = match client.query_opt(
            "SELECT id, name FROM dns_zones WHERE name = $1",
            &[&zone_name],
        )? {
            Some(row) => row,
            None => return Ok(None),
        };

        let zone_id: i32 = zone_row.get(0);
        let name: String = zone_row.get(1);

        let records = client.query(
            "SELECT name, rtype, ttl, data FROM dns_records WHERE zone_id = $1",
            &[&zone_id],
        )?;

        let mut zone_records = Vec::new();
        for record_row in records {
            zone_records.push(PersistedRecord {
                name: record_row.get(0),
                rtype: record_row.get(1),
                ttl: record_row.get(2),
                data: record_row.get(3),
            });
        }

        Ok(Some(PersistedZone {
            name,
            records: zone_records,
        }))
    }

    async fn save_zone(&self, zone: &PersistedZone) -> Result<(), Box<dyn Error>> {
        let mut client = self.get_connection()?;

        client.execute(
            "INSERT INTO dns_zones (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET updated_at = CURRENT_TIMESTAMP",
            &[&zone.name],
        )?;

        Ok(())
    }

    async fn delete_zone(&self, zone_name: &str) -> Result<(), Box<dyn Error>> {
        let mut client = self.get_connection()?;

        client.execute("DELETE FROM dns_zones WHERE name = $1", &[&zone_name])?;

        Ok(())
    }

    async fn save_record(
        &self,
        zone_name: &str,
        record: &PersistedRecord,
    ) -> Result<(), Box<dyn Error>> {
        let mut client = self.get_connection()?;

        client.execute(
            "INSERT INTO dns_zones (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
            &[&zone_name],
        )?;

        client.execute(
            "INSERT INTO dns_records (zone_id, name, rtype, ttl, data)
             SELECT id, $2, $3, $4, $5 FROM dns_zones WHERE name = $1
             ON CONFLICT (zone_id, name, rtype) DO UPDATE
             SET ttl = $4, data = $5, updated_at = CURRENT_TIMESTAMP",
            &[
                &zone_name,
                &record.name,
                &record.rtype,
                &(record.ttl as i32),
                &record.data,
            ],
        )?;

        Ok(())
    }

    async fn delete_record(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut client = self.get_connection()?;

        client.execute(
            "DELETE FROM dns_records
             WHERE zone_id = (SELECT id FROM dns_zones WHERE name = $1)
             AND name = $2 AND rtype = $3",
            &[&zone_name, &name, &rtype],
        )?;

        Ok(())
    }
}

// In-memory implementation for testing or when no database is available
//
// This backend stores zones in memory using a HashMap. It's useful for:
// - Unit and integration testing
// - Development environments without a database
// - Temporary zone caching
//
// Note: Zones are lost when the server restarts.
//
// # Example
//
// ```
// use hackflare_dns::ns::MemoryPersistence;
//
// let storage = MemoryPersistence::new();
// # tokio::runtime::Runtime::new().unwrap().block_on(async {
// // storage is ready to use immediately
// # });
// ```
pub struct MemoryPersistence {
    zones: parking_lot::RwLock<HashMap<String, PersistedZone>>,
}

impl MemoryPersistence {
    // Create a new in-memory persistence backend
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
        let mut zones = self.zones.write();
        zones.insert(zone.name.clone(), zone.clone());
        Ok(())
    }

    async fn delete_zone(&self, zone_name: &str) -> Result<(), Box<dyn Error>> {
        let mut zones = self.zones.write();
        zones.remove(zone_name);
        Ok(())
    }

    async fn save_record(
        &self,
        zone_name: &str,
        record: &PersistedRecord,
    ) -> Result<(), Box<dyn Error>> {
        let mut zones = self.zones.write();
        zones
            .entry(zone_name.to_string())
            .or_insert_with(|| PersistedZone {
                name: zone_name.to_string(),
                records: Vec::new(),
            })
            .records
            .push(record.clone());
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
