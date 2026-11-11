use crate::DnsError;
use crate::dns::DnsConfig;
use crate::dns::packet::{Edns, Message, Protocol, Rcode};
use crate::dns::rr::{RData, Record, RecordType, canonical_name};
use crate::ns::persistence::{PersistedRecord, PersistedZone, ZonePersistence};
use crate::ns::zone::Zone;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// SOA serial floor for today in UTC, formatted as `YYYYMMDD00`.
///
/// The serial grows monotonically within a day; the two trailing digits are a
/// zero-padded per-day revision counter, matching the conventional
/// date-stamped SOA serial scheme.
#[must_use]
pub fn today_serial_base() -> u32 {
    use chrono::Datelike;
    let now = chrono::Utc::now();
    (now.year() as u32) * 1_000_000 + now.month() * 10_000 + now.day() * 100
}

/// Provides authoritative DNS zone management using the built-in in-memory zone store.
pub struct AuthorityStore {
    config: DnsConfig,
    pub zones: RwLock<HashMap<String, Zone>>,
    persistence: Option<Arc<dyn ZonePersistence>>,
}

impl AuthorityStore {
    pub fn new(config: DnsConfig) -> Self {
        Self {
            config,
            zones: RwLock::new(HashMap::new()),
            persistence: None,
        }
    }

    /// Create `AuthorityStore` with persistence enabled
    pub fn with_persistence(config: DnsConfig, persistence: Arc<dyn ZonePersistence>) -> Self {
        Self {
            config,
            zones: RwLock::new(HashMap::new()),
            persistence: Some(persistence),
        }
    }

    /// Create a new DNS zone with default SOA record.
    pub async fn create_zone(&self, name: impl Into<String>) -> bool {
        let serial = today_serial_base();
        self.create_zone_with_serial(name, serial).await
    }

    /// Create a new DNS zone, seeding its SOA record with the given serial.
    async fn create_zone_with_serial(&self, name: impl Into<String>, serial: u32) -> bool {
        let Some((zone_key, zone)) = self.build_zone(&name.into(), serial, &[]) else {
            return false;
        };

        self.zones.write().await.insert(zone_key.clone(), zone);

        if let Some(persistence) = &self.persistence {
            let persisted = PersistedZone {
                name: zone_key.clone(),
                serial,
                records: Vec::new(),
            };
            if let Err(e) = persistence.save_zone(&persisted).await {
                eprintln!("[hackflare:dns] failed to persist zone {zone_key}: {e}");
            }
        }

        true
    }

    /// Delete an existing DNS zone.
    pub async fn delete_zone(&self, name: &str) -> bool {
        let zone_name = Self::normalize_zone_name(name);
        let zone_key = zone_name.trim_end_matches('.').to_string();
        let removed = self.zones.write().await.remove(&zone_key).is_some();

        if removed
            && let Some(persistence) = &self.persistence
            && let Err(e) = persistence.delete_zone(&zone_key).await
        {
            eprintln!("[hackflare:dns] failed to delete zone {zone_key} from storage: {e}");
        }
        removed
    }

    /// Add a DNS record to a zone.
    pub async fn add_record(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
        ttl: u32,
        data: &str,
    ) -> bool {
        self.add_record_inner(zone_name, name, rtype, ttl, data)
            .await
    }

    /// Add a DNS record to a zone.
    async fn add_record_inner(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
        ttl: u32,
        data: &str,
    ) -> bool {
        let normalized_zone = Self::normalize_zone_name(zone_name);
        let zone_key = normalized_zone.trim_end_matches('.').to_string();

        let fqdn = Self::build_fqdn(&normalized_zone, name);
        let Some(record_name) = canonical_name(&fqdn) else {
            return false;
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };
        let Some(rdata) = RData::parse(record_type, data) else {
            return false;
        };

        let record = Record {
            name: record_name,
            ttl,
            data: rdata,
        };

        let (ok, serial) = {
            let mut zones = self.zones.write().await;
            let Some(zone) = zones.get_mut(&zone_key) else {
                return false;
            };
            let ok = zone.upsert(record);
            let serial = ok.then(|| zone.bump_serial(today_serial_base()));
            (ok, serial)
        };

        if let Some(serial) = serial {
            self.persist_serial(&zone_key, serial).await;
        }

        if ok && let Some(persistence) = &self.persistence {
            let record = PersistedRecord {
                name: name.to_string(),
                rtype: rtype.to_string(),
                ttl,
                data: data.to_string(),
            };
            if let Err(e) = persistence.save_record(zone_name, &record).await {
                eprintln!(
                    "[hackflare:dns] failed to persist record {name} ({rtype}) in zone {zone_name}: {e}"
                );
            }
        }

        ok
    }

    /// Remove a DNS record from a zone.
    pub async fn remove_record(&self, zone_name: &str, name: &str, rtype: &str) -> bool {
        let normalized_zone = Self::normalize_zone_name(zone_name);
        let zone_key = normalized_zone.trim_end_matches('.').to_string();

        let fqdn = Self::build_fqdn(&normalized_zone, name);
        let Some(record_name) = canonical_name(&fqdn) else {
            return false;
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };

        // Apex NS records are managed by the authoritative server; refuse deletion.
        if record_type == RecordType::Ns && record_name == normalized_zone {
            return false;
        }

        let (removed, serial) = {
            let mut zones = self.zones.write().await;
            let Some(zone) = zones.get_mut(&zone_key) else {
                return false;
            };
            let removed = zone.remove(&record_name, record_type);
            let serial = removed.then(|| zone.bump_serial(today_serial_base()));
            (removed, serial)
        };

        if let Some(serial) = serial {
            self.persist_serial(&zone_key, serial).await;
        }

        if removed
            && let Some(persistence) = &self.persistence
            && let Err(e) = persistence
                .delete_record(zone_name, name, rtype.trim())
                .await
        {
            eprintln!("[hackflare:dns] failed to remove record {name} ({rtype}) from storage: {e}");
        }

        removed
    }

    /// Remove a single DNS record value from a zone.
    pub async fn remove_record_value(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
        data: &str,
    ) -> bool {
        let normalized_zone = Self::normalize_zone_name(zone_name);
        let zone_key = normalized_zone.trim_end_matches('.').to_string();

        let fqdn = Self::build_fqdn(&normalized_zone, name);
        let Some(record_name) = canonical_name(&fqdn) else {
            return false;
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };
        let Some(rdata) = RData::parse(record_type, data) else {
            return false;
        };

        // Apex NS records are managed by the authoritative server; refuse deletion.
        if record_type == RecordType::Ns && record_name == normalized_zone {
            return false;
        }

        let (removed, serial) = {
            let mut zones = self.zones.write().await;
            let Some(zone) = zones.get_mut(&zone_key) else {
                return false;
            };
            let removed = zone.remove_value(&record_name, record_type, &rdata);
            let serial = removed.then(|| zone.bump_serial(today_serial_base()));
            (removed, serial)
        };

        if let Some(serial) = serial {
            self.persist_serial(&zone_key, serial).await;
        }

        if removed
            && let Some(persistence) = &self.persistence
            && let Err(e) = persistence
                .delete_record_value(zone_name, name, rtype.trim(), data)
                .await
        {
            eprintln!(
                "[hackflare:dns] failed to remove record {name} ({rtype}) value from storage: {e}"
            );
        }

        removed
    }

    /// Replace a single DNS record value, preserving its storage identity
    pub async fn update_record_value(
        &self,
        zone_name: &str,
        old: &PersistedRecord,
        new: &PersistedRecord,
    ) -> bool {
        let normalized_zone = Self::normalize_zone_name(zone_name);
        let zone_key = normalized_zone.trim_end_matches('.').to_string();

        let old_fqdn = Self::build_fqdn(&normalized_zone, &old.name);
        let Some(old_record_name) = canonical_name(&old_fqdn) else {
            return false;
        };
        let Ok(old_record_type) = RecordType::from_str(old.rtype.trim()) else {
            return false;
        };
        let Some(old_rdata) = RData::parse(old_record_type, &old.data) else {
            return false;
        };

        let new_fqdn = Self::build_fqdn(&normalized_zone, &new.name);
        let Some(new_record_name) = canonical_name(&new_fqdn) else {
            return false;
        };
        let Ok(new_record_type) = RecordType::from_str(new.rtype.trim()) else {
            return false;
        };
        let Some(new_rdata) = RData::parse(new_record_type, &new.data) else {
            return false;
        };

        // Apex NS records are managed by the authoritative server; refuse updates
        if old_record_type == RecordType::Ns && old_record_name == normalized_zone {
            return false;
        }

        let record = Record {
            name: new_record_name,
            ttl: new.ttl,
            data: new_rdata,
        };

        let (updated, serial) = {
            let mut zones = self.zones.write().await;
            let Some(zone) = zones.get_mut(&zone_key) else {
                return false;
            };
            if !zone.remove_value(&old_record_name, old_record_type, &old_rdata) {
                return false;
            }
            let updated = zone.upsert(record);
            let serial = updated.then(|| zone.bump_serial(today_serial_base()));
            (updated, serial)
        };

        if !updated {
            return false;
        }

        if let Some(serial) = serial {
            self.persist_serial(&zone_key, serial).await;
        }

        if let Some(persistence) = &self.persistence
            && let Err(e) = persistence.update_record(zone_name, old, new).await
        {
            eprintln!(
                "[hackflare:dns] failed to update record {} ({}) in zone {zone_name}: {e}",
                new.name, new.rtype
            );
        }

        true
    }

    /// List all zones.
    pub async fn list_zones(&self) -> Vec<String> {
        self.zones.read().await.keys().cloned().collect()
    }

    /// Check if a zone exists for the given name.
    pub async fn contains_zone_for(&self, name: &str) -> bool {
        let Some(name) = canonical_name(name) else {
            return false;
        };
        let zones = self.zones.read().await;
        find_zone_key(&zones, &name).is_some()
    }

    /// Return the zone name that best matches the given name, if any.
    pub async fn find_zone_name(&self, name: &str) -> Option<String> {
        let name = canonical_name(name)?;
        let zones = self.zones.read().await;
        find_zone_key(&zones, &name)
    }

    /// Load all zones from persistence storage.
    ///
    /// Loading never writes to storage, so starting an instance does not disturb zones served by other instances.
    pub async fn load_zones_from_storage(&self) -> Result<(), DnsError> {
        let Some(persistence) = &self.persistence else {
            return Err(DnsError::PersistenceUnconfigured);
        };

        let zones = persistence
            .load_zones()
            .await
            .map_err(|e| DnsError::PersistenceOperation(format!("{e}")))?;

        let mut loaded = HashMap::with_capacity(zones.len());
        for zone in zones {
            if let Some((zone_key, built)) = self.build_zone(&zone.name, zone.serial, &zone.records)
            {
                loaded.insert(zone_key, built);
            }
        }

        *self.zones.write().await = loaded;
        Ok(())
    }

    // === Helper Methods ===

    /// Persist a zone's new SOA serial after an in-memory mutation.
    async fn persist_serial(&self, zone_key: &str, serial: u32) {
        let Some(persistence) = &self.persistence else {
            return;
        };
        if let Err(e) = persistence.update_zone_serial(zone_key, serial).await {
            eprintln!(
                "[hackflare:dns] failed to persist SOA serial {serial} for zone {zone_key}: {e}"
            );
        }
    }

    /// Normalize zone name (lowercase, trailing dot).
    pub fn normalize_zone_name(name: &str) -> String {
        let trimmed = name.trim().trim_end_matches('.').to_ascii_lowercase();
        if trimmed.is_empty() {
            ".".to_string()
        } else {
            format!("{trimmed}.")
        }
    }

    /// Build fully-qualified domain name from zone and relative name.
    pub fn build_fqdn(zone: &str, name: &str) -> String {
        let normalized = name.trim().trim_end_matches('.').to_ascii_lowercase();

        if normalized.is_empty() || normalized == "@" {
            zone.to_string()
        } else if normalized.ends_with(zone.trim_end_matches('.')) {
            format!("{normalized}.")
        } else {
            format!("{normalized}.{zone}")
        }
    }

    // Build a zone from persisted data without reading or writing storage.
    fn build_zone(
        &self,
        name: &str,
        serial: u32,
        records: &[PersistedRecord],
    ) -> Option<(String, Zone)> {
        let zone_name = Self::normalize_zone_name(name);
        let origin = canonical_name(&zone_name)?;

        let mut zone = Zone::new(origin.clone(), serial);
        zone.upsert(self.build_soa_record(&origin, serial)?);

        // Advertise the configured nameservers at the zone apex so that authoritative NS queries resolve to Hackflare's own servers.
        for ns_name in &self.config.nameservers {
            let Some(ns_name) = canonical_name(ns_name) else {
                eprintln!("[hackflare:dns] skipping invalid nameserver in config");
                continue;
            };
            zone.upsert(Record {
                name: origin.clone(),
                ttl: self.config.soa_ttl,
                data: RData::Ns(ns_name),
            });
        }

        for record in records {
            let fqdn = Self::build_fqdn(&zone_name, &record.name);
            let Some(record_name) = canonical_name(&fqdn) else {
                continue;
            };
            let Ok(record_type) = RecordType::from_str(record.rtype.trim()) else {
                continue;
            };
            let Some(rdata) = RData::parse(record_type, &record.data) else {
                continue;
            };
            zone.upsert(Record {
                name: record_name,
                ttl: record.ttl,
                data: rdata,
            });
        }

        Some((zone_name.trim_end_matches('.').to_string(), zone))
    }

    // Build a default SOA record for a zone with the given serial.
    fn build_soa_record(&self, origin: &str, serial: u32) -> Option<Record> {
        let mname = canonical_name(&self.config.soa_mname)?;
        let rname = canonical_name(&self.config.soa_rname)?;

        Some(Record {
            name: origin.to_string(),
            ttl: self.config.soa_ttl,
            data: RData::Soa {
                mname,
                rname,
                serial,
                refresh: self.config.soa_refresh,
                retry: self.config.soa_retry,
                expire: self.config.soa_expire,
                minimum: self.config.soa_minimum,
            },
        })
    }
}

// Zone lookup & query handling

/// Find the best-matching zone for a canonical name.
#[must_use]
pub fn find_zone<'a>(zones: &'a HashMap<String, Zone>, name: &str) -> Option<&'a Zone> {
    let key = find_zone_key(zones, name)?;
    zones.get(&key)
}

fn find_zone_key(zones: &HashMap<String, Zone>, name: &str) -> Option<String> {
    let mut current = name.trim_end_matches('.').to_string();
    loop {
        if zones.contains_key(&current) {
            return Some(current);
        }
        let (_, suffix) = current.split_once('.')?;
        if suffix.is_empty() {
            return None;
        }
        current = suffix.to_string();
    }
}

fn error_response(
    query: &Message,
    rcode: Rcode,
    recursion_available: bool,
    server_payload_size: u16,
) -> Message {
    let mut response = Message::response_to(query);
    response.header.set_authoritative(true);
    response.header.set_recursion_available(recursion_available);
    response.header.set_rcode(rcode);
    response.edns = query
        .edns
        .as_ref()
        .map(|edns| Edns::response_to(edns, server_payload_size));
    response
}

impl AuthorityStore {
    /// Answer an authoritative query, including zone transfers.
    pub async fn answer(&self, query: &Message, src: SocketAddr, protocol: Protocol) -> Message {
        let Some(question) = query.questions.first() else {
            return error_response(
                query,
                Rcode::FormErr,
                self.config.recursion_enabled,
                self.config.max_edns_payload_size,
            );
        };

        let Some(query_name) = canonical_name(&question.name) else {
            return error_response(
                query,
                Rcode::FormErr,
                self.config.recursion_enabled,
                self.config.max_edns_payload_size,
            );
        };

        let qtype = RecordType::from_u16(question.qtype);
        let zones = self.zones.read().await;

        let Some(zone) = find_zone(&zones, &query_name) else {
            return error_response(
                query,
                Rcode::Refused,
                self.config.recursion_enabled,
                self.config.max_edns_payload_size,
            );
        };

        if matches!(qtype, RecordType::Axfr | RecordType::Ixfr) {
            if protocol != Protocol::Tcp || !self.config.transfer_allow.contains(&src.ip()) {
                return error_response(
                    query,
                    Rcode::Refused,
                    self.config.recursion_enabled,
                    self.config.max_edns_payload_size,
                );
            }

            // IXFR history is not retained yet, so RFC-compliant fallback is a full AXFR.
            // Transfer responses are returned as one TCP message.
            let soa = zone
                .all_records()
                .find(|record| record.record_type() == RecordType::Soa)
                .cloned();
            let Some(soa) = soa else {
                return error_response(
                    query,
                    Rcode::ServFail,
                    self.config.recursion_enabled,
                    self.config.max_edns_payload_size,
                );
            };

            let mut records: Vec<&Record> = zone
                .all_records()
                .filter(|record| record.record_type() != RecordType::Soa)
                .collect();
            records.sort_by(|a, b| {
                a.name
                    .cmp(&b.name)
                    .then_with(|| a.record_type().cmp(&b.record_type()))
            });

            let mut answers = Vec::with_capacity(records.len() + 2);
            answers.push(soa.to_wire());
            answers.extend(records.iter().map(|record| record.to_wire()));
            answers.push(soa.to_wire());

            let mut response = Message::response_to(query);
            response.header.set_authoritative(true);
            response
                .header
                .set_recursion_available(self.config.recursion_enabled);
            response.header.set_rcode(Rcode::NoError);
            response.answers = answers;
            response.edns = query
                .edns
                .as_ref()
                .map(|edns| Edns::response_to(edns, self.config.max_edns_payload_size));
            return response;
        }

        let lookup = if qtype == RecordType::Soa {
            zone.lookup(zone.origin(), RecordType::Soa)
        } else {
            zone.lookup(&query_name, qtype)
        };

        let mut response = Message::response_to(query);
        response.header.set_authoritative(true);
        response
            .header
            .set_recursion_available(self.config.recursion_enabled);
        response.edns = query
            .edns
            .as_ref()
            .map(|edns| Edns::response_to(edns, self.config.max_edns_payload_size));

        if lookup.nx_domain {
            response.header.set_rcode(Rcode::NxDomain);
            return response;
        }

        response.header.set_rcode(Rcode::NoError);
        response
            .additionals
            .extend(lookup.additionals.iter().map(Record::to_wire));

        let is_referral = lookup.records.first().is_some_and(|record| {
            record.record_type() == RecordType::Ns
                && qtype != RecordType::Ns
                && qtype != RecordType::Any
        });

        if is_referral {
            response
                .authorities
                .extend(lookup.records.iter().map(Record::to_wire));
        } else {
            response
                .answers
                .extend(lookup.records.iter().map(Record::to_wire));
        }

        response
    }
}
