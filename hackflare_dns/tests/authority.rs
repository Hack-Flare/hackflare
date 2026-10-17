#![allow(clippy::unwrap_used)]

use hackflare_dns::ns::authority::*;

use hackflare_dns::dns::config::DnsConfig;
use hackflare_dns::ns::persistence::{
    MemoryPersistence, PersistedRecord, PersistedZone, ZonePersistence,
};
use hickory_server::net::runtime::TokioRuntimeProvider;
use hickory_server::proto::op::Metadata;
use hickory_server::proto::op::{LowerQuery, MessageType, OpCode, Query, ResponseCode};
use hickory_server::proto::rr::{LowerName, Name, RecordType};
use hickory_server::store::in_memory::InMemoryZoneHandler;
use hickory_server::zone_handler::{AxfrPolicy, LookupOptions, ZoneHandler, ZoneType};
use std::sync::Arc;

#[tokio::test]
async fn build_auth_response_marks_recursion_when_enabled() {
    let handler = InMemoryZoneHandler::<TokioRuntimeProvider>::empty(
        Name::from_utf8("example.com.").unwrap(),
        ZoneType::Primary,
        AxfrPolicy::Deny,
    );
    let meta = Metadata::new(7, MessageType::Query, OpCode::Query);
    let query: LowerQuery = Query::new().into();
    let message = build_auth_response(None, &handler, &meta, &query, true).await;
    assert!(message.metadata.authoritative);
    assert!(message.metadata.recursion_available);
    assert_eq!(message.metadata.response_code, ResponseCode::ServFail);
}

#[tokio::test]
async fn build_auth_response_omits_recursion_when_disabled() {
    let handler = InMemoryZoneHandler::<TokioRuntimeProvider>::empty(
        Name::from_utf8("example.com.").unwrap(),
        ZoneType::Primary,
        AxfrPolicy::Deny,
    );
    let meta = Metadata::new(7, MessageType::Query, OpCode::Query);
    let query: LowerQuery = Query::new().into();
    let message = build_auth_response(None, &handler, &meta, &query, false).await;
    assert!(message.metadata.authoritative);
    assert!(!message.metadata.recursion_available);
}

#[tokio::test]
async fn create_zone_succeeds() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    let result = store.create_zone("example.com").await;
    assert!(result);

    let zones = store.list_zones().await;
    assert!(zones.contains(&"example.com".to_string()));
}

#[tokio::test]
async fn create_zone_normalizes_names() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    let result = store.create_zone("Example.COM.").await;
    assert!(result);

    let zones = store.list_zones().await;
    assert!(zones.contains(&"example.com".to_string()));
}

#[tokio::test]
async fn delete_zone_succeeds() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    store.create_zone("example.com").await;
    let result = store.delete_zone("example.com").await;
    assert!(result);

    let zones = store.list_zones().await;
    assert!(!zones.contains(&"example.com".to_string()));
}

#[tokio::test]
async fn add_record_creates_valid_a_record() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    store.create_zone("example.com").await;
    let result = store
        .add_record("example.com", "www", "A", 300, "192.168.1.1")
        .await;
    assert!(result);
}

#[tokio::test]
async fn add_record_fails_for_nonexistent_zone() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    let result = store
        .add_record("nonexistent.com", "www", "A", 300, "192.168.1.1")
        .await;
    assert!(!result);
}

#[tokio::test]
async fn add_record_allows_multiple_values_same_name_type() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    store.create_zone("example.com").await;

    // Two A records on the same hostname co-exist
    assert!(
        store
            .add_record("example.com", "www", "A", 300, "192.168.1.1")
            .await
    );
    assert!(
        store
            .add_record("example.com", "www", "A", 300, "192.168.1.2")
            .await
    );

    // and two TXT records on _acme-challenge (wildcard + root dns01) co-exist.
    assert!(
        store
            .add_record("example.com", "_acme-challenge", "TXT", 60, "challenge-one")
            .await
    );
    assert!(
        store
            .add_record("example.com", "_acme-challenge", "TXT", 60, "challenge-two")
            .await
    );

    let zones = store.list_zones().await;
    assert!(zones.contains(&"example.com".to_string()));
}

#[tokio::test]
async fn add_record_with_various_types() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    store.create_zone("example.com").await;

    assert!(
        store
            .add_record("example.com", "www", "A", 300, "192.168.1.1")
            .await
    );
    assert!(
        store
            .add_record("example.com", "mail", "MX", 300, "10 mail.example.com")
            .await
    );
    assert!(
        store
            .add_record("example.com", "@", "TXT", 300, "v=spf1 ~all")
            .await
    );
}

#[tokio::test]
async fn remove_record_succeeds() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    store.create_zone("example.com").await;
    store
        .add_record("example.com", "www", "A", 300, "192.168.1.1")
        .await;

    let result = store.remove_record("example.com", "www", "A").await;
    assert!(result);
}

#[tokio::test]
async fn remove_record_fails_for_nonexistent() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    store.create_zone("example.com").await;

    let result = store.remove_record("example.com", "nonexistent", "A").await;
    assert!(!result);
}

#[tokio::test]
async fn remove_record_value_removes_only_matching_value() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    store.create_zone("example.com").await;
    store
        .add_record("example.com", "www", "A", 300, "192.168.1.1")
        .await;
    store
        .add_record("example.com", "www", "A", 300, "192.168.1.2")
        .await;

    let result = store
        .remove_record_value("example.com", "www", "A", "192.168.1.1")
        .await;
    assert!(result);

    // The other value remains unpublished/queryable.
    let name = LowerName::new(&Name::from_utf8("www.example.com.").unwrap());
    {
        let handler =
            find_zone(&*store.zones.read().await, &name).expect("example.com handler exists");
        let records = handler.records().await;
        let mut remaining: Vec<String> = records
            .values()
            .flat_map(|set| {
                set.records_without_rrsigs()
                    .map(|r| r.data.to_string())
                    .collect::<Vec<_>>()
            })
            .filter(|d| d.contains("192.168.1."))
            .collect();
        remaining.sort();
        assert_eq!(remaining, vec!["192.168.1.2"]);
    }

    // Removing the other value as well empties the RRset.
    let result = store
        .remove_record_value("example.com", "www", "A", "192.168.1.2")
        .await;
    assert!(result);
}

#[tokio::test]
async fn remove_record_value_succeeds_for_acme_challenge_duplicates() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    store.create_zone("example.com").await;
    store
        .add_record("example.com", "_acme-challenge", "TXT", 60, "challenge-one")
        .await;
    store
        .add_record("example.com", "_acme-challenge", "TXT", 60, "challenge-two")
        .await;

    // Deleting one TXT leaves the other in place (wildcard + root dns01).
    assert!(
        store
            .remove_record_value("example.com", "_acme-challenge", "TXT", "challenge-one")
            .await
    );

    let name = LowerName::new(&Name::from_utf8("_acme-challenge.example.com.").unwrap());
    let handler = find_zone(&*store.zones.read().await, &name).expect("handler exists");
    let records = handler.records().await;
    let remaining: Vec<String> = records
        .values()
        .flat_map(|set| {
            set.records_without_rrsigs()
                .map(|r| r.data.to_string())
                .collect::<Vec<_>>()
        })
        .filter(|d| d.contains("challenge"))
        .collect();
    assert_eq!(remaining, vec!["challenge-two"]);
}

#[test]
fn normalize_zone_name_adds_trailing_dot() {
    assert_eq!(
        AuthorityStore::normalize_zone_name("example.com"),
        "example.com."
    );
    assert_eq!(
        AuthorityStore::normalize_zone_name("example.com."),
        "example.com."
    );
}

#[test]
fn normalize_zone_name_lowercases() {
    assert_eq!(
        AuthorityStore::normalize_zone_name("Example.COM"),
        "example.com."
    );
}

#[test]
fn build_fqdn_handles_apex() {
    assert_eq!(
        AuthorityStore::build_fqdn("example.com.", "@"),
        "example.com."
    );
    assert_eq!(
        AuthorityStore::build_fqdn("example.com.", ""),
        "example.com."
    );
}

#[test]
fn build_fqdn_constructs_subdomain() {
    assert_eq!(
        AuthorityStore::build_fqdn("example.com.", "www"),
        "www.example.com."
    );
}

#[test]
fn build_fqdn_handles_fully_qualified() {
    assert_eq!(
        AuthorityStore::build_fqdn("example.com.", "www.example.com"),
        "www.example.com."
    );
}

#[tokio::test]
async fn contains_zone_for_returns_true_for_existing_zone() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    store.create_zone("example.com").await;

    let name = LowerName::new(&Name::from_utf8("example.com.").unwrap());
    assert!(store.contains_zone_for(&name).await);
}

#[tokio::test]
async fn contains_zone_for_returns_false_for_missing_zone() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    let name = LowerName::new(&Name::from_utf8("nonexistent.com.").unwrap());
    assert!(!store.contains_zone_for(&name).await);
}

#[tokio::test]
async fn create_zone_adds_apex_ns_records() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config.clone());
    store.create_zone("example.com").await;

    let handler = store.zones.read().await.get("example.com").unwrap().clone();
    let records = handler.records().await;
    let apex = Name::from_utf8("example.com.").unwrap();
    let mut apex_ns: Vec<String> = records
        .values()
        .filter(|set| set.record_type() == RecordType::NS && set.name() == &apex)
        .flat_map(|set| {
            set.records_without_rrsigs()
                .map(|r| r.data.to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    apex_ns.sort();
    assert_eq!(
        apex_ns,
        vec![
            "ns1.hackflare.net.".to_string(),
            "ns2.hackflare.net.".to_string()
        ]
    );
}

#[tokio::test]
async fn apex_ns_records_resolve_on_lookup() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);
    store.create_zone("example.com").await;

    let handler = store.zones.read().await.get("example.com").unwrap().clone();
    let apex = LowerName::new(&Name::from_utf8("example.com.").unwrap());
    let lookup = handler
        .lookup(&apex, RecordType::NS, None, LookupOptions::default())
        .await
        .map_result()
        .unwrap()
        .unwrap();
    let mut served: Vec<String> = lookup.iter().map(|r| r.data.to_string()).collect();
    served.sort();
    assert_eq!(
        served,
        vec![
            "ns1.hackflare.net.".to_string(),
            "ns2.hackflare.net.".to_string()
        ]
    );
}

#[tokio::test]
async fn apex_ns_records_are_protected_from_removal() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);
    store.create_zone("example.com").await;

    assert!(!store.remove_record("example.com", "@", "NS").await);
    assert!(
        !store
            .remove_record_value("example.com", "@", "NS", "ns1.hackflare.net.")
            .await
    );

    // Non-apex NS records (subdomain delegations) remain removable.
    assert!(
        store
            .add_record("example.com", "sub", "NS", 3600, "ns1.example.com")
            .await
    );
    assert!(store.remove_record("example.com", "sub", "NS").await);
}

#[tokio::test]
async fn load_zones_from_storage_readds_apex_ns() {
    let persistence = Arc::new(MemoryPersistence::new());
    let zone = PersistedZone {
        name: "example.com".to_string(),
        serial: 42,
        records: Vec::new(),
    };
    persistence.save_zone(&zone).await.unwrap();

    let store = AuthorityStore::with_persistence(DnsConfig::default_config(), persistence);
    store.load_zones_from_storage().await.unwrap();

    let handler = store.zones.read().await.get("example.com").unwrap().clone();
    let apex = Name::from_utf8("example.com.").unwrap();
    let records = handler.records().await;
    let apex_ns: Vec<String> = records
        .values()
        .filter(|set| set.record_type() == RecordType::NS && set.name() == &apex)
        .flat_map(|set| {
            set.records_without_rrsigs()
                .map(|r| r.data.to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(apex_ns.len(), 2);
}

#[tokio::test]
async fn serial_starts_at_todays_base_on_zone_creation() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);
    store.create_zone("example.com").await;

    let handler = store.zones.read().await.get("example.com").unwrap().clone();
    assert_eq!(handler.serial().await, today_serial_base());
}

#[tokio::test]
async fn serial_increments_on_add_and_remove() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);
    store.create_zone("example.com").await;

    let serial = || async {
        let handler = store.zones.read().await.get("example.com").unwrap().clone();
        handler.serial().await
    };

    assert!(
        store
            .add_record("example.com", "www", "A", 300, "192.168.1.1")
            .await
    );
    assert_eq!(serial().await, today_serial_base().saturating_add(1));

    assert!(store.remove_record("example.com", "www", "A").await);
    assert_eq!(serial().await, today_serial_base().saturating_add(2));
}

#[tokio::test]
async fn serial_increments_on_single_value_removal() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);
    store.create_zone("example.com").await;

    store
        .add_record("example.com", "www", "A", 300, "192.168.1.1")
        .await;
    store
        .add_record("example.com", "www", "A", 300, "192.168.1.2")
        .await;

    let serial = || async {
        let handler = store.zones.read().await.get("example.com").unwrap().clone();
        handler.serial().await
    };

    let before = serial().await;
    assert!(
        store
            .remove_record_value("example.com", "www", "A", "192.168.1.1")
            .await
    );
    assert_eq!(serial().await, before.saturating_add(1));
}

#[tokio::test]
async fn load_zones_from_storage_restores_serial_without_bumping() {
    let persistence = Arc::new(MemoryPersistence::new());
    let stored_serial = 2024010100;
    let zone = PersistedZone {
        name: "example.com".to_string(),
        serial: stored_serial,
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
    };
    persistence.save_zone(&zone).await.unwrap();

    let store = AuthorityStore::with_persistence(DnsConfig::default_config(), persistence.clone());
    store.load_zones_from_storage().await.unwrap();

    let handler = store.zones.read().await.get("example.com").unwrap().clone();
    assert_eq!(handler.serial().await, stored_serial);

    // a subsequent mutation bumps from the restored value, never below today's date
    assert!(
        store
            .add_record("example.com", "api", "A", 300, "192.168.1.2")
            .await
    );
    assert_eq!(
        handler.serial().await,
        stored_serial.saturating_add(1).max(today_serial_base())
    );

    let persisted = persistence.load_zone("example.com").await.unwrap().unwrap();
    assert_eq!(
        persisted.serial,
        stored_serial.saturating_add(1).max(today_serial_base())
    );
}
