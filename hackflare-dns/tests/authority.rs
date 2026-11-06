#![allow(clippy::unwrap_used)]

use hackflare_dns::ns::authority::*;

use hackflare_dns::dns::config::DnsConfig;
use hackflare_dns::dns::packet::{self, Message, Rcode};
use hackflare_dns::dns::rr::{Record, RecordType};
use hackflare_dns::ns::persistence::{
    MemoryPersistence, PersistedRecord, PersistedZone, ZonePersistence,
};
use std::sync::Arc;

async fn zone_records(store: &AuthorityStore, zone_name: &str) -> Vec<Record> {
    let zones = store.zones.read().await;
    zones
        .get(zone_name)
        .map(|zone| zone.all_records().cloned().collect())
        .unwrap_or_default()
}

#[tokio::test]
async fn answer_marks_recursion_when_enabled() {
    let mut config = DnsConfig::default_config();
    config.recursion_enabled = true;
    let store = AuthorityStore::new(config);
    store.create_zone("example.com").await;
    store
        .add_record("example.com", "www", "A", 300, "192.0.2.1")
        .await;

    let query = Message::query(7, "www.example.com", packet::TYPE_A);
    let response = store
        .answer(
            &query,
            "127.0.0.1:5300".parse().unwrap(),
            packet::Protocol::Udp,
        )
        .await;

    assert!(response.header.authoritative());
    assert!(response.header.recursion_available());
    assert_eq!(response.header.rcode(), Rcode::NoError);
}

#[tokio::test]
async fn answer_omits_recursion_when_disabled() {
    let store = AuthorityStore::new(DnsConfig::default_config());
    store.create_zone("example.com").await;

    let query = Message::query(7, "www.example.com", packet::TYPE_A);
    let response = store
        .answer(
            &query,
            "127.0.0.1:5300".parse().unwrap(),
            packet::Protocol::Udp,
        )
        .await;

    assert!(response.header.authoritative());
    assert!(!response.header.recursion_available());
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

    let records = zone_records(&store, "example.com").await;
    let www_a: Vec<String> = records
        .iter()
        .filter(|record| record.name == "www.example.com." && record.record_type() == RecordType::A)
        .map(|record| record.data.to_string())
        .collect();
    assert_eq!(www_a.len(), 2);

    let txt: Vec<String> = records
        .iter()
        .filter(|record| {
            record.name == "_acme-challenge.example.com." && record.record_type() == RecordType::Txt
        })
        .map(|record| record.data.to_string())
        .collect();
    assert_eq!(txt.len(), 2);
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

    // The other value remains published and queryable.
    let records = zone_records(&store, "example.com").await;
    let mut remaining: Vec<String> = records
        .iter()
        .filter(|record| record.name == "www.example.com.")
        .map(|record| record.data.to_string())
        .collect();
    remaining.sort();
    assert_eq!(remaining, vec!["192.168.1.2"]);

    // Removing the other value as well empties the RRset.
    let result = store
        .remove_record_value("example.com", "www", "A", "192.168.1.2")
        .await;
    assert!(result);

    let records = zone_records(&store, "example.com").await;
    assert!(
        !records
            .iter()
            .any(|record| record.name == "www.example.com.")
    );
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

    let records = zone_records(&store, "example.com").await;
    let remaining: Vec<String> = records
        .iter()
        .filter(|record| record.name == "_acme-challenge.example.com.")
        .map(|record| record.data.to_string())
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

    assert!(store.contains_zone_for("example.com.").await);
}

#[tokio::test]
async fn contains_zone_for_returns_false_for_missing_zone() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);

    assert!(!store.contains_zone_for("nonexistent.com.").await);
}

#[tokio::test]
async fn create_zone_adds_apex_ns_records() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config.clone());
    store.create_zone("example.com").await;

    let records = zone_records(&store, "example.com").await;
    let mut apex_ns: Vec<String> = records
        .iter()
        .filter(|record| record.name == "example.com." && record.record_type() == RecordType::Ns)
        .map(|record| record.data.to_string())
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

    let zones = store.zones.read().await;
    let zone = zones.get("example.com").unwrap();
    let lookup = zone.lookup("example.com.", RecordType::Ns);
    let mut served: Vec<String> = lookup
        .records
        .iter()
        .map(|record| record.data.to_string())
        .collect();
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

    let records = zone_records(&store, "example.com").await;
    let apex_ns: Vec<String> = records
        .iter()
        .filter(|record| record.name == "example.com." && record.record_type() == RecordType::Ns)
        .map(|record| record.data.to_string())
        .collect();
    assert_eq!(apex_ns.len(), 2);
}

#[tokio::test]
async fn serial_starts_at_todays_base_on_zone_creation() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);
    store.create_zone("example.com").await;

    let zones = store.zones.read().await;
    assert_eq!(
        zones.get("example.com").unwrap().serial(),
        today_serial_base()
    );
}

#[tokio::test]
async fn serial_increments_on_add_and_remove() {
    let config = DnsConfig::default_config();
    let store = AuthorityStore::new(config);
    store.create_zone("example.com").await;

    let serial = || async {
        let zones = store.zones.read().await;
        zones.get("example.com").unwrap().serial()
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
        let zones = store.zones.read().await;
        zones.get("example.com").unwrap().serial()
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

    let zones = store.zones.read().await;
    assert_eq!(zones.get("example.com").unwrap().serial(), stored_serial);
    drop(zones);

    // a subsequent mutation bumps from the restored value, never below today's date
    assert!(
        store
            .add_record("example.com", "api", "A", 300, "192.168.1.2")
            .await
    );
    let zones = store.zones.read().await;
    assert_eq!(
        zones.get("example.com").unwrap().serial(),
        stored_serial.saturating_add(1).max(today_serial_base())
    );
    drop(zones);

    let persisted = persistence.load_zone("example.com").await.unwrap().unwrap();
    assert_eq!(
        persisted.serial,
        stored_serial.saturating_add(1).max(today_serial_base())
    );
}

#[tokio::test]
async fn answer_returns_authoritative_a_record() {
    let store = AuthorityStore::new(DnsConfig::default_config());
    store.create_zone("example.com").await;
    store
        .add_record("example.com", "www", "A", 300, "192.0.2.1")
        .await;

    let query = Message::query(1, "www.example.com", packet::TYPE_A);
    let response = store
        .answer(
            &query,
            "127.0.0.1:5300".parse().unwrap(),
            packet::Protocol::Udp,
        )
        .await;

    assert_eq!(response.header.rcode(), Rcode::NoError);
    assert!(response.header.authoritative());
    assert_eq!(response.answers.len(), 1);
    assert_eq!(response.answers[0].rdata, vec![192, 0, 2, 1]);
}

#[tokio::test]
async fn answer_returns_nxdomain_for_missing_name() {
    let store = AuthorityStore::new(DnsConfig::default_config());
    store.create_zone("example.com").await;

    let query = Message::query(1, "missing.example.com", packet::TYPE_A);
    let response = store
        .answer(
            &query,
            "127.0.0.1:5300".parse().unwrap(),
            packet::Protocol::Udp,
        )
        .await;

    assert_eq!(response.header.rcode(), Rcode::NxDomain);
}

#[tokio::test]
async fn answer_refuses_unknown_zone() {
    let store = AuthorityStore::new(DnsConfig::default_config());

    let query = Message::query(1, "example.com", packet::TYPE_A);
    let response = store
        .answer(
            &query,
            "127.0.0.1:5300".parse().unwrap(),
            packet::Protocol::Udp,
        )
        .await;

    assert_eq!(response.header.rcode(), Rcode::Refused);
}

#[tokio::test]
async fn answer_refuses_zonetransfer_over_udp() {
    let store = AuthorityStore::new(DnsConfig::default_config());
    store.create_zone("example.com").await;

    let query = Message::query(1, "example.com", packet::TYPE_AXFR);
    let response = store
        .answer(
            &query,
            "127.0.0.1:5300".parse().unwrap(),
            packet::Protocol::Udp,
        )
        .await;

    assert_eq!(response.header.rcode(), Rcode::Refused);
}

#[tokio::test]
async fn answer_allows_zonetransfer_over_tcp_for_allowed_client() {
    let mut config = DnsConfig::default_config();
    config.transfer_allow.push("127.0.0.1".parse().unwrap());
    let store = AuthorityStore::new(config);
    store.create_zone("example.com").await;
    store
        .add_record("example.com", "www", "A", 300, "192.0.2.1")
        .await;

    let query = Message::query(1, "example.com", packet::TYPE_AXFR);
    let response = store
        .answer(
            &query,
            "127.0.0.1:5300".parse().unwrap(),
            packet::Protocol::Tcp,
        )
        .await;

    assert_eq!(response.header.rcode(), Rcode::NoError);
    assert!(response.answers.len() >= 3);
    assert_eq!(response.answers[0].rtype, packet::TYPE_SOA);
    assert_eq!(response.answers.last().unwrap().rtype, packet::TYPE_SOA);
}
