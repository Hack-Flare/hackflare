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

/// SQL schema for PostgreSQL persistence backend.
///
/// The consumer crate must ensure these tables exist before creating a
/// [`PostgresPersistence`] backend. The exact SQL is also listed as a reference
/// in the [`PostgresPersistence`] documentation.
pub const POSTGRES_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS dns_zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS dns_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_id UUID NOT NULL REFERENCES dns_zones(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    rtype TEXT NOT NULL,
    ttl INTEGER NOT NULL DEFAULT 300,
    data TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(zone_id, name, rtype)
);
"#;

/// PostgreSQL-backed persistence for DNS zones and records.
///
/// The consumer crate is responsible for creating and managing the
/// [`sqlx::PgPool`] — the pool is passed into [`new`](Self::new) and
/// `PostgresPersistence` holds a reference to it.
///
/// # Expected schema
///
/// The following tables must exist (created by the consumer's migrations):
///
/// ```sql
/// CREATE TABLE dns_zones (
///     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
///     name TEXT NOT NULL UNIQUE,
///     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
///     updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
/// );
///
/// CREATE TABLE dns_records (
///     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
///     zone_id UUID NOT NULL REFERENCES dns_zones(id) ON DELETE CASCADE,
///     name TEXT NOT NULL,
///     rtype TEXT NOT NULL,
///     ttl INTEGER NOT NULL DEFAULT 300,
///     data TEXT NOT NULL,
///     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
///     updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
///     UNIQUE(zone_id, name, rtype)
/// );
/// ```
///
/// # Example (consumer side)
///
/// ```rust,no_run
/// use hackflare_dns::ns::PostgresPersistence;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = sqlx::PgPool::connect("postgres://...").await?;
///
/// let persistence: Arc<dyn hackflare_dns::ns::ZonePersistence> =
///     Arc::new(PostgresPersistence::new(pool));
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct PostgresPersistence {
    pool: sqlx::PgPool,
}

impl PostgresPersistence {
    /// Create a new PostgreSQL persistence backend from an existing pool.
    ///
    /// The consumer crate manages the pool lifecycle and must ensure the
    /// expected schema is in place before calling methods on this backend.
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ZonePersistence for PostgresPersistence {
    async fn load_zones(&self) -> Result<Vec<PersistedZone>, Box<dyn Error>> {
        #[derive(sqlx::FromRow)]
        struct ZoneRecordRow {
            zone_name: String,
            record_name: Option<String>,
            rtype: Option<String>,
            ttl: Option<i32>,
            data: Option<String>,
        }

        let rows = sqlx::query_as::<_, ZoneRecordRow>(
            r#"
            SELECT z.name AS zone_name,
                   r.name AS record_name,
                   r.rtype,
                   r.ttl,
                   r.data
            FROM dns_zones z
            LEFT JOIN dns_records r ON r.zone_id = z.id
            ORDER BY z.name, r.name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut zone_map: HashMap<String, PersistedZone> = HashMap::new();
        for row in rows {
            let entry = zone_map.entry(row.zone_name.clone()).or_insert_with(|| {
                PersistedZone {
                    name: row.zone_name,
                    records: Vec::new(),
                }
            });
            if let (Some(name), Some(rtype), Some(ttl), Some(data)) =
                (row.record_name, row.rtype, row.ttl, row.data)
            {
                entry.records.push(PersistedRecord {
                    name,
                    rtype,
                    ttl: ttl as u32,
                    data,
                });
            }
        }

        Ok(zone_map.into_values().collect())
    }

    async fn load_zone(&self, zone_name: &str) -> Result<Option<PersistedZone>, Box<dyn Error>> {
        let zone_row: Option<(String,)> =
            sqlx::query_as("SELECT name FROM dns_zones WHERE name = $1")
                .bind(zone_name)
                .fetch_optional(&self.pool)
                .await?;

        let Some((name,)) = zone_row else {
            return Ok(None);
        };

        let records = sqlx::query_as::<_, (String, String, i32, String)>(
            r#"
            SELECT r.name, r.rtype, r.ttl, r.data
            FROM dns_records r
            JOIN dns_zones z ON z.id = r.zone_id
            WHERE z.name = $1
            ORDER BY r.name
            "#,
        )
        .bind(zone_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(PersistedZone {
            name,
            records: records
                .into_iter()
                .map(|(n, rt, ttl, d)| PersistedRecord {
                    name: n,
                    rtype: rt,
                    ttl: ttl as u32,
                    data: d,
                })
                .collect(),
        }))
    }

    async fn save_zone(&self, zone: &PersistedZone) -> Result<(), Box<dyn Error>> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO dns_zones (name) VALUES ($1)
            ON CONFLICT (name) DO UPDATE SET updated_at = now()
            "#,
        )
        .bind(&zone.name)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM dns_records
            WHERE zone_id = (SELECT id FROM dns_zones WHERE name = $1)
            "#,
        )
        .bind(&zone.name)
        .execute(&mut *tx)
        .await?;

        for record in &zone.records {
            sqlx::query(
                r#"
                INSERT INTO dns_records (zone_id, name, rtype, ttl, data)
                VALUES (
                    (SELECT id FROM dns_zones WHERE name = $1),
                    $2, $3, $4, $5
                )
                "#,
            )
            .bind(&zone.name)
            .bind(&record.name)
            .bind(&record.rtype)
            .bind(record.ttl as i32)
            .bind(&record.data)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn delete_zone(&self, zone_name: &str) -> Result<(), Box<dyn Error>> {
        sqlx::query("DELETE FROM dns_zones WHERE name = $1")
            .bind(zone_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn save_record(
        &self,
        zone_name: &str,
        record: &PersistedRecord,
    ) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            r#"
            INSERT INTO dns_records (zone_id, name, rtype, ttl, data)
            VALUES (
                (SELECT id FROM dns_zones WHERE name = $1),
                $2, $3, $4, $5
            )
            ON CONFLICT (zone_id, name, rtype)
            DO UPDATE SET ttl = EXCLUDED.ttl, data = EXCLUDED.data, updated_at = now()
            "#,
        )
        .bind(zone_name)
        .bind(&record.name)
        .bind(&record.rtype)
        .bind(record.ttl as i32)
        .bind(&record.data)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_record(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
    ) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            r#"
            DELETE FROM dns_records
            WHERE zone_id = (SELECT id FROM dns_zones WHERE name = $1)
              AND name = $2
              AND rtype = $3
            "#,
        )
        .bind(zone_name)
        .bind(name)
        .bind(rtype)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
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

    // ── PostgreSQL persistence tests (require DATABASE_URL env) ──

    /// Connect to a test database from `DATABASE_URL` and apply the schema.
    /// Returns `None` if the env var is unset or the connection fails.
    async fn get_test_pool() -> Option<sqlx::PgPool> {
        let url = std::env::var("DATABASE_URL").ok()?;
        let pool = sqlx::PgPool::connect(&url).await.ok()?;
        sqlx::query(POSTGRES_SCHEMA_SQL).execute(&pool).await.ok()?;
        // clean slate — cascade clears records first
        sqlx::query("TRUNCATE dns_records, dns_zones RESTART IDENTITY CASCADE")
            .execute(&pool)
            .await
            .ok()?;
        Some(pool)
    }

    fn test_zone(name: &str) -> PersistedZone {
        PersistedZone {
            name: name.to_string(),
            records: vec![
                PersistedRecord {
                    name: "www".to_string(),
                    rtype: "A".to_string(),
                    ttl: 300,
                    data: "192.168.1.1".to_string(),
                },
                PersistedRecord {
                    name: "mail".to_string(),
                    rtype: "MX".to_string(),
                    ttl: 600,
                    data: "10 mail.example.com".to_string(),
                },
            ],
        }
    }

    #[tokio::test]
    async fn postgres_save_zone_then_load_zones() {
        let Some(pool) = get_test_pool().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };
        let storage = PostgresPersistence::new(pool);

        storage.save_zone(&test_zone("example.com")).await.unwrap();
        storage.save_zone(&test_zone("other.org")).await.unwrap();

        let zones = storage.load_zones().await.unwrap();
        assert_eq!(zones.len(), 2);

        let names: Vec<&str> = zones.iter().map(|z| z.name.as_str()).collect();
        assert!(names.contains(&"example.com"));
        assert!(names.contains(&"other.org"));
    }

    #[tokio::test]
    async fn postgres_load_zone_by_name() {
        let Some(pool) = get_test_pool().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };
        let storage = PostgresPersistence::new(pool);

        storage.save_zone(&test_zone("example.com")).await.unwrap();

        let zone = storage
            .load_zone("example.com")
            .await
            .unwrap()
            .expect("zone should exist");
        assert_eq!(zone.name, "example.com");
        assert_eq!(zone.records.len(), 2);
    }

    #[tokio::test]
    async fn postgres_load_zone_returns_none_for_missing() {
        let Some(pool) = get_test_pool().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };
        let storage = PostgresPersistence::new(pool);

        let zone = storage.load_zone("nonexistent.com").await.unwrap();
        assert!(zone.is_none());
    }

    #[tokio::test]
    async fn postgres_delete_zone_removes_it() {
        let Some(pool) = get_test_pool().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };
        let storage = PostgresPersistence::new(pool);

        storage.save_zone(&test_zone("example.com")).await.unwrap();
        storage.delete_zone("example.com").await.unwrap();

        let zone = storage.load_zone("example.com").await.unwrap();
        assert!(zone.is_none());
    }

    #[tokio::test]
    async fn postgres_save_record_upserts() {
        let Some(pool) = get_test_pool().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };
        let storage = PostgresPersistence::new(pool);

        storage.save_zone(&test_zone("example.com")).await.unwrap();

        // add a new record
        let extra = PersistedRecord {
            name: "api".to_string(),
            rtype: "A".to_string(),
            ttl: 120,
            data: "10.0.0.1".to_string(),
        };
        storage.save_record("example.com", &extra).await.unwrap();

        let zone = storage.load_zone("example.com").await.unwrap().unwrap();
        assert_eq!(zone.records.len(), 3);

        // upsert — same name + rtype, different ttl/data
        let updated = PersistedRecord {
            name: "api".to_string(),
            rtype: "A".to_string(),
            ttl: 999,
            data: "10.0.0.2".to_string(),
        };
        storage.save_record("example.com", &updated).await.unwrap();

        let zone = storage.load_zone("example.com").await.unwrap().unwrap();
        assert_eq!(zone.records.len(), 3);
        let rec = zone
            .records
            .iter()
            .find(|r| r.name == "api")
            .expect("api record should exist");
        assert_eq!(rec.ttl, 999);
        assert_eq!(rec.data, "10.0.0.2");
    }

    #[tokio::test]
    async fn postgres_delete_record_removes_it() {
        let Some(pool) = get_test_pool().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };
        let storage = PostgresPersistence::new(pool);

        storage.save_zone(&test_zone("example.com")).await.unwrap();
        storage
            .delete_record("example.com", "www", "A")
            .await
            .unwrap();

        let zone = storage.load_zone("example.com").await.unwrap().unwrap();
        assert_eq!(zone.records.len(), 1);
        assert_eq!(zone.records[0].name, "mail");
    }

    #[tokio::test]
    async fn postgres_transactional_save_zone() {
        let Some(pool) = get_test_pool().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };
        let storage = PostgresPersistence::new(pool);

        storage.save_zone(&test_zone("example.com")).await.unwrap();

        // replace all records atomically
        let replacement = PersistedZone {
            name: "example.com".to_string(),
            records: vec![PersistedRecord {
                name: "blog".to_string(),
                rtype: "CNAME".to_string(),
                ttl: 300,
                data: "proxy.example.com".to_string(),
            }],
        };
        storage.save_zone(&replacement).await.unwrap();

        let zone = storage.load_zone("example.com").await.unwrap().unwrap();
        assert_eq!(zone.records.len(), 1);
        assert_eq!(zone.records[0].name, "blog");
        assert_eq!(zone.records[0].rtype, "CNAME");
    }

    #[tokio::test]
    async fn postgres_load_zones_empty_db() {
        let Some(pool) = get_test_pool().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };
        let storage = PostgresPersistence::new(pool);

        let zones = storage.load_zones().await.unwrap();
        assert!(zones.is_empty());
    }
}
