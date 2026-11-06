#![allow(clippy::unwrap_used)]

use hackflare_dns::ns::persistence::*;

#[tokio::test]
async fn memory_persistence_saves_and_loads_zones() {
    let storage = MemoryPersistence::new();

    let zone = PersistedZone {
        name: "example.com".to_string(),
        serial: 2024010101,
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
    assert_eq!(loaded.clone().unwrap().name, "example.com");
    assert_eq!(loaded.unwrap().serial, 2024010101);
}

#[tokio::test]
async fn memory_persistence_updates_zone_serial() {
    let storage = MemoryPersistence::new();

    let zone = PersistedZone {
        name: "example.com".to_string(),
        serial: 2024010101,
        records: vec![],
    };
    storage.save_zone(&zone).await.unwrap();

    storage
        .update_zone_serial("example.com", 2024010102)
        .await
        .unwrap();

    let loaded = storage.load_zone("example.com").await.unwrap().unwrap();
    assert_eq!(loaded.serial, 2024010102);
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
        serial: 2024010101,
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
        serial: 2024010101,
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

#[tokio::test]
async fn memory_persistence_deletes_single_value_only() {
    let storage = MemoryPersistence::new();

    let zone = PersistedZone {
        name: "example.com".to_string(),
        serial: 2024010101,
        records: vec![],
    };

    let first = PersistedRecord {
        name: "www".to_string(),
        rtype: "A".to_string(),
        ttl: 300,
        data: "192.168.1.1".to_string(),
    };
    let second = PersistedRecord {
        name: "www".to_string(),
        rtype: "A".to_string(),
        ttl: 300,
        data: "192.168.1.2".to_string(),
    };

    storage.save_zone(&zone).await.unwrap();
    storage.save_record("example.com", &first).await.unwrap();
    storage.save_record("example.com", &second).await.unwrap();

    storage
        .delete_record_value("example.com", "www", "A", "192.168.1.1")
        .await
        .unwrap();

    let zone = storage.load_zone("example.com").await.unwrap().unwrap();
    let remaining: Vec<_> = zone
        .records
        .iter()
        .filter(|r| r.name == "www")
        .cloned()
        .collect();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].data, "192.168.1.2");
}

#[tokio::test]
async fn memory_persistence_delete_value_is_idempotent() {
    let storage = MemoryPersistence::new();

    let zone = PersistedZone {
        name: "example.com".to_string(),
        serial: 2024010101,
        records: vec![PersistedRecord {
            name: "www".to_string(),
            rtype: "A".to_string(),
            ttl: 300,
            data: "192.168.1.1".to_string(),
        }],
    };
    storage.save_zone(&zone).await.unwrap();

    storage
        .delete_record_value("example.com", "www", "A", "192.168.1.1")
        .await
        .unwrap();
    storage
        .delete_record_value("example.com", "www", "A", "192.168.1.1")
        .await
        .unwrap();

    let zone = storage.load_zone("example.com").await.unwrap().unwrap();
    assert_eq!(zone.records.len(), 0);
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
        serial: 2024010101,
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
async fn postgres_update_zone_serial() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("skipping: DATABASE_URL not set or unreachable");
        return;
    };
    let storage = PostgresPersistence::new(pool);

    storage.save_zone(&test_zone("example.com")).await.unwrap();

    storage
        .update_zone_serial("example.com", 2024010199)
        .await
        .unwrap();

    let loaded = storage.load_zone("example.com").await.unwrap().unwrap();
    assert_eq!(loaded.serial, 2024010199);
    assert_eq!(loaded.records.len(), 2);
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
async fn postgres_save_record_stores_duplicate_values() {
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

    // duplicate - same name + rtype, different data is stored as its own row
    let second = PersistedRecord {
        name: "api".to_string(),
        rtype: "A".to_string(),
        ttl: 120,
        data: "10.0.0.2".to_string(),
    };
    storage.save_record("example.com", &second).await.unwrap();

    let zone = storage.load_zone("example.com").await.unwrap().unwrap();
    assert_eq!(zone.records.len(), 4);
    let api_records: Vec<_> = zone
        .records
        .iter()
        .filter(|r| r.name == "api")
        .cloned()
        .collect();
    assert_eq!(api_records.len(), 2);
    assert!(api_records.iter().any(|r| r.data == "10.0.0.1"));
    assert!(api_records.iter().any(|r| r.data == "10.0.0.2"));

    // identical data - ttl is refreshed in place, no new row
    let refreshed = PersistedRecord {
        name: "api".to_string(),
        rtype: "A".to_string(),
        ttl: 999,
        data: "10.0.0.2".to_string(),
    };
    storage
        .save_record("example.com", &refreshed)
        .await
        .unwrap();

    let zone = storage.load_zone("example.com").await.unwrap().unwrap();
    assert_eq!(zone.records.len(), 4);
    let rec = zone
        .records
        .iter()
        .find(|r| r.name == "api" && r.data == "10.0.0.2")
        .expect("api record should exist");
    assert_eq!(rec.ttl, 999);
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
async fn postgres_delete_record_value_removes_only_one() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("skipping: DATABASE_URL not set or unreachable");
        return;
    };
    let storage = PostgresPersistence::new(pool);

    storage.save_zone(&test_zone("example.com")).await.unwrap();

    let first = PersistedRecord {
        name: "www".to_string(),
        rtype: "A".to_string(),
        ttl: 300,
        data: "192.168.1.1".to_string(),
    };
    let second = PersistedRecord {
        name: "www".to_string(),
        rtype: "A".to_string(),
        ttl: 300,
        data: "192.168.1.2".to_string(),
    };
    storage.save_record("example.com", &first).await.unwrap();
    storage.save_record("example.com", &second).await.unwrap();

    storage
        .delete_record_value("example.com", "www", "A", "192.168.1.1")
        .await
        .unwrap();

    let zone = storage.load_zone("example.com").await.unwrap().unwrap();
    assert_eq!(zone.records.len(), 3);
    let www: Vec<_> = zone
        .records
        .iter()
        .filter(|r| r.name == "www")
        .cloned()
        .collect();
    assert_eq!(www.len(), 1);
    assert_eq!(www[0].data, "192.168.1.2");
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
        serial: 2024010101,
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
