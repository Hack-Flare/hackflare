use crate::DnsError;
use crate::dns::DnsConfig;
use crate::dns::packet::{self, Edns, Message, Protocol, Rcode};
use crate::dns::wire::encode_name_labels_vec;
use crate::ns::persistence::{PersistedRecord, PersistedZone, ZonePersistence};
use hickory_server::net::runtime::TokioRuntimeProvider;
use hickory_server::proto::op::ResponseCode;
use hickory_server::proto::rr::{
    LowerName, Name, RData, Record, RecordType,
    rdata::{NS, SOA},
};
use hickory_server::proto::serialize::binary::{BinEncodable, BinEncoder, NameEncoding};
use hickory_server::store::in_memory::InMemoryZoneHandler;
use hickory_server::zone_handler::{AxfrPolicy, LookupError, LookupOptions, ZoneHandler, ZoneType};
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

/// Provides authoritative DNS zone management using hickory-server's in-memory zones.
pub struct AuthorityStore {
    config: DnsConfig,
    pub zones: RwLock<HashMap<String, Arc<InMemoryZoneHandler<TokioRuntimeProvider>>>>,
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
        let zone_name = Self::normalize_zone_name(&name.into());
        let Ok(origin) = Name::from_utf8(&zone_name) else {
            return false;
        };

        let Some(soa_record) = self.build_soa_record(&origin, serial) else {
            return false;
        };

        let handler = Arc::new(InMemoryZoneHandler::empty(
            origin.clone(),
            ZoneType::Primary,
            AxfrPolicy::Deny,
        ));

        if !handler.upsert(soa_record, serial).await {
            return false;
        }

        // Advertise the configured nameservers at the zone apex so that
        // authoritative NS queries resolve to Hackflare's own servers.
        for ns_name in &self.config.nameservers {
            let Ok(ns_name) = Name::from_utf8(ns_name.trim()) else {
                eprintln!("[hackflare:dns] skipping invalid nameserver in config");
                continue;
            };
            handler
                .upsert(
                    Record::from_rdata(origin.clone(), self.config.soa_ttl, RData::NS(NS(ns_name))),
                    serial,
                )
                .await;
        }

        let zone_key = zone_name.trim_end_matches('.').to_string();
        self.zones.write().await.insert(zone_key.clone(), handler);

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
        self.add_record_inner(zone_name, name, rtype, ttl, data, true)
            .await
    }

    /// Add a DNS record to a zone, optionally bumping the SOA serial on success.
    async fn add_record_inner(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
        ttl: u32,
        data: &str,
        bump: bool,
    ) -> bool {
        let normalized_zone = Self::normalize_zone_name(zone_name);
        let zone_key = normalized_zone.trim_end_matches('.').to_string();

        let Some(handler) = self.zones.read().await.get(&zone_key).cloned() else {
            return false;
        };

        let fqdn = Self::build_fqdn(&normalized_zone, name);
        let Ok(record_name) = Name::from_utf8(&fqdn) else {
            return false;
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };
        let Ok(rdata) = RData::try_from_str(record_type, data) else {
            return false;
        };

        let serial = handler.serial().await;
        let ok = handler
            .upsert(Record::from_rdata(record_name, ttl, rdata), serial)
            .await;

        if ok && bump {
            self.bump_soa_serial(&zone_key).await;
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

        let Some(handler) = self.zones.read().await.get(&zone_key).cloned() else {
            return false;
        };

        let fqdn = Self::build_fqdn(&normalized_zone, name);
        let record_name = match Name::from_utf8(&fqdn) {
            Ok(n) => n,
            Err(_) => return false,
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };

        // Apex NS records are managed by the authoritative server; refuse deletion
        if record_type == RecordType::NS && fqdn == normalized_zone {
            return false;
        }

        let removed = {
            let mut records = handler.records_mut().await;
            let before_count = records.len();
            let target_name = LowerName::new(&record_name);
            records
                .retain(|key, _| !(key.name() == &target_name && key.record_type == record_type));
            before_count != records.len()
        };

        if removed {
            self.bump_soa_serial(&zone_key).await;
            if let Some(persistence) = &self.persistence
                && let Err(e) = persistence
                    .delete_record(zone_name, name, rtype.trim())
                    .await
            {
                eprintln!(
                    "[hackflare:dns] failed to remove record {name} ({rtype}) from storage: {e}"
                );
            }
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

        let Some(handler) = self.zones.read().await.get(&zone_key).cloned() else {
            return false;
        };

        let fqdn = Self::build_fqdn(&normalized_zone, name);
        let Ok(record_name) = Name::from_utf8(&fqdn) else {
            return false;
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };
        let Ok(rdata) = RData::try_from_str(record_type, data) else {
            return false;
        };

        // Apex NS records are managed by the authoritative server; refuse deletion.
        if record_type == RecordType::NS && fqdn == normalized_zone {
            return false;
        }

        let target = Record::from_rdata(record_name.clone(), 0, rdata);
        let target_name = LowerName::new(&record_name);
        let serial = handler.serial().await;

        let removed = {
            let mut records = handler.records_mut().await;
            let key = records
                .keys()
                .find(|k| k.name() == &target_name && k.record_type == record_type)
                .cloned();

            let Some(key) = key else {
                return false;
            };

            let Some(current) = records.get(&key).cloned() else {
                return false;
            };

            // Clone the RecordSet out of the Arc so we can drop a single value.
            let mut new_set = Arc::try_unwrap(current).unwrap_or_else(|arc| (*arc).clone());
            if !new_set.remove(&target, serial) {
                return false;
            }

            if new_set.is_empty() {
                records.remove(&key).is_some()
            } else {
                records.insert(key, Arc::new(new_set));
                true
            }
        };

        if removed {
            self.bump_soa_serial(&zone_key).await;
            if let Some(persistence) = &self.persistence
                && let Err(e) = persistence
                    .delete_record_value(zone_name, name, rtype.trim(), data)
                    .await
            {
                eprintln!(
                    "[hackflare:dns] failed to remove record {name} ({rtype}) value from storage: {e}"
                );
            }
        }

        removed
    }

    /// List all zones.
    pub async fn list_zones(&self) -> Vec<String> {
        self.zones.read().await.keys().cloned().collect()
    }

    /// Check if a zone exists for the given name.
    pub async fn contains_zone_for(&self, name: &LowerName) -> bool {
        let zones = self.zones.read().await;
        find_zone(&zones, name).is_some()
    }

    /// Return the zone name that best matches the given name, if any.
    pub async fn find_zone_name(&self, name: &LowerName) -> Option<String> {
        let zones = self.zones.read().await;
        let mut current = name.clone();
        loop {
            let key = current.to_utf8().trim_end_matches('.').to_string();
            if zones.contains_key(&key) {
                return Some(key);
            }
            if current.is_root() {
                return None;
            }
            current = current.base_name();
        }
    }

    /// Load all zones from persistence storage.
    pub async fn load_zones_from_storage(&self) -> Result<(), DnsError> {
        let Some(persistence) = &self.persistence else {
            return Err(DnsError::PersistenceUnconfigured);
        };

        let zones = persistence
            .load_zones()
            .await
            .map_err(|e| DnsError::PersistenceOperation(format!("{e}")))?;

        for zone in zones {
            self.create_zone_with_serial(&zone.name, zone.serial).await;
            for record in zone.records {
                let _ = self
                    .add_record_inner(
                        &zone.name,
                        &record.name,
                        &record.rtype,
                        record.ttl,
                        &record.data,
                        false,
                    )
                    .await;
            }
        }

        Ok(())
    }

    // === Helper Methods ===

    /// Increment a zone's SOA serial and persist the new value.
    ///
    /// The new serial is stamped with the current UTC date and is always
    /// greater than the previous serial: `max(current + 1, today_serial_base())`.
    /// This keeps secondaries in sync while avoiding serial collapse on a new day.
    async fn bump_soa_serial(&self, zone_key: &str) -> Option<u32> {
        let handler = self.zones.read().await.get(zone_key)?.clone();
        let current = handler.serial().await;
        let new_serial = current.saturating_add(1).max(today_serial_base());

        let origin: Name = handler.origin().clone().into();
        // Replace the SOA record; hickory accepts the update because the new
        // serial is strictly greater than the existing one.
        let soa_record = self.build_soa_record(&origin, new_serial)?;
        if !handler.upsert(soa_record, new_serial).await {
            return None;
        }

        if let Some(persistence) = &self.persistence
            && let Err(e) = persistence.update_zone_serial(zone_key, new_serial).await
        {
            eprintln!(
                "[hackflare:dns] failed to persist SOA serial {new_serial} for zone {zone_key}: {e}"
            );
        }

        Some(new_serial)
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

    // Build a default SOA record for a zone with the given serial.
    #[allow(clippy::cast_possible_wrap)]
    fn build_soa_record(&self, origin: &Name, serial: u32) -> Option<Record> {
        let mname = Name::from_utf8(&self.config.soa_mname).ok()?;
        let rname = Name::from_utf8(&self.config.soa_rname).ok()?;

        let soa = SOA::new(
            mname,
            rname,
            serial,
            self.config.soa_refresh as i32,
            self.config.soa_retry as i32,
            self.config.soa_expire as i32,
            self.config.soa_minimum,
        );

        Some(Record::from_rdata(
            origin.clone(),
            self.config.soa_ttl,
            RData::SOA(soa),
        ))
    }
}

// Zone lookup & query handling

/// Find the best-matching zone handler by walking parent suffixes.
pub fn find_zone(
    zones: &HashMap<String, Arc<InMemoryZoneHandler<TokioRuntimeProvider>>>,
    name: &LowerName,
) -> Option<Arc<InMemoryZoneHandler<TokioRuntimeProvider>>> {
    let mut current = name.clone();
    loop {
        let key = current.to_utf8().trim_end_matches('.').to_string();
        if let Some(handler) = zones.get(&key) {
            return Some(handler.clone());
        }
        if current.is_root() {
            return None;
        }
        current = current.base_name();
    }
}

/// Encode RDATA for records stored in the authoritative zone.
fn encode_rdata(data: &RData) -> Vec<u8> {
    match data {
        RData::A(address) => address.0.octets().to_vec(),
        RData::AAAA(address) => address.0.octets().to_vec(),
        RData::NS(name) => encode_name_labels_vec(&name.0.to_utf8()),
        RData::CNAME(name) => encode_name_labels_vec(&name.0.to_utf8()),
        RData::PTR(name) => encode_name_labels_vec(&name.0.to_utf8()),
        RData::ANAME(name) => encode_name_labels_vec(&name.0.to_utf8()),
        RData::MX(mx) => {
            let mut out = mx.preference.to_be_bytes().to_vec();
            out.extend_from_slice(&encode_name_labels_vec(&mx.exchange.to_utf8()));
            out
        }
        RData::TXT(txt) => {
            let mut out = Vec::new();
            for value in &txt.txt_data {
                let len = value.len().min(usize::from(u8::MAX));
                out.push(len as u8);
                out.extend_from_slice(&value[..len]);
            }
            out
        }
        RData::SOA(soa) => {
            let mut out = encode_name_labels_vec(&soa.mname.to_utf8());
            out.extend_from_slice(&encode_name_labels_vec(&soa.rname.to_utf8()));
            out.extend_from_slice(&soa.serial.to_be_bytes());
            out.extend_from_slice(&soa.refresh.to_be_bytes());
            out.extend_from_slice(&soa.retry.to_be_bytes());
            out.extend_from_slice(&soa.expire.to_be_bytes());
            out.extend_from_slice(&soa.minimum.to_be_bytes());
            out
        }
        RData::SRV(srv) => {
            let mut out = srv.priority.to_be_bytes().to_vec();
            out.extend_from_slice(&srv.weight.to_be_bytes());
            out.extend_from_slice(&srv.port.to_be_bytes());
            out.extend_from_slice(&encode_name_labels_vec(&srv.target.to_utf8()));
            out
        }
        RData::CAA(caa) => {
            let mut out = Vec::with_capacity(2 + caa.tag.len() + caa.value.len());
            let flags = u8::from(caa.issuer_critical) << 7;
            out.push(flags | caa.reserved_flags);
            out.push(caa.tag.len().min(usize::from(u8::MAX)) as u8);
            out.extend_from_slice(&caa.tag.as_bytes()[..caa.tag.len().min(usize::from(u8::MAX))]);
            out.extend_from_slice(&caa.value);
            out
        }
        other => {
            let mut buffer = Vec::new();
            let mut encoder = BinEncoder::new(&mut buffer);
            encoder.set_name_encoding(NameEncoding::Uncompressed);
            if other.emit(&mut encoder).is_ok() {
                buffer
            } else {
                Vec::new()
            }
        }
    }
}

/// Convert a hickory resource record into a wire record with uncompressed RDATA.
fn record_to_wire(record: &Record) -> packet::Record {
    packet::Record {
        name: record.name.to_utf8(),
        rtype: u16::from(record.record_type()),
        class: u16::from(record.dns_class),
        ttl: record.ttl,
        rdata: encode_rdata(&record.data),
        rdata_offset: 0,
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

/// Build a response message for an authoritative zone lookup.
pub fn build_auth_response(
    result: Option<Result<hickory_server::zone_handler::AuthLookup, LookupError>>,
    query: &Message,
    recursion_available: bool,
    server_payload_size: u16,
) -> Message {
    let mut response = Message::response_to(query);
    response.header.set_authoritative(true);
    response.header.set_recursion_available(recursion_available);
    response.edns = query
        .edns
        .as_ref()
        .map(|edns| Edns::response_to(edns, server_payload_size));

    let Some(result) = result else {
        response.header.set_rcode(Rcode::ServFail);
        return response;
    };

    match result {
        Ok(mut auth_lookup) => {
            response.header.set_rcode(Rcode::NoError);
            if let Some(adds) = auth_lookup.take_additionals() {
                response.additionals.extend(adds.iter().map(record_to_wire));
            }

            let question = query.questions.first();
            let is_referral = auth_lookup.iter().next().is_some_and(|record| {
                record.record_type() == RecordType::NS
                    && question.is_some_and(|question| {
                        question.qtype != packet::TYPE_NS && question.qtype != packet::TYPE_ANY
                    })
            });

            let records: Vec<packet::Record> = auth_lookup.iter().map(record_to_wire).collect();
            if is_referral {
                response.authorities.extend(records);
            } else {
                response.answers.extend(records);
            }
            response
        }
        Err(error) => {
            if error.is_nx_domain() {
                response.header.set_rcode(Rcode::NxDomain);
            } else if let LookupError::ResponseCode(code) = &error
                && (*code == ResponseCode::Refused || *code == ResponseCode::NotAuth)
            {
                response
                    .header
                    .set_rcode(Rcode::from_u8(u16::from(*code) as u8));
            } else {
                response.header.set_rcode(Rcode::NoError);
            }
            response
        }
    }
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

        let Ok(mut query_name) = Name::from_utf8(&question.name) else {
            return error_response(
                query,
                Rcode::FormErr,
                self.config.recursion_enabled,
                self.config.max_edns_payload_size,
            );
        };
        query_name.set_fqdn(true);
        let query_name = LowerName::new(&query_name);

        let handler = {
            let zones = self.zones.read().await;
            find_zone(&zones, &query_name)
        };

        let Some(handler) = handler else {
            return error_response(
                query,
                Rcode::Refused,
                self.config.recursion_enabled,
                self.config.max_edns_payload_size,
            );
        };

        if matches!(question.qtype, packet::TYPE_AXFR | packet::TYPE_IXFR) {
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
            let mut records = Vec::new();
            {
                let sets = handler.records().await;
                for set in sets.values() {
                    records.extend(set.records_without_rrsigs().cloned());
                }
            }
            let soa = records
                .iter()
                .find(|record| record.record_type() == RecordType::SOA)
                .cloned();
            let Some(soa) = soa else {
                return error_response(
                    query,
                    Rcode::ServFail,
                    self.config.recursion_enabled,
                    self.config.max_edns_payload_size,
                );
            };
            records.retain(|record| record.record_type() != RecordType::SOA);
            records.sort_by(|a, b| {
                a.name
                    .cmp(&b.name)
                    .then_with(|| a.record_type().cmp(&b.record_type()))
            });

            let mut answers = Vec::with_capacity(records.len() + 2);
            answers.push(record_to_wire(&soa));
            answers.extend(records.iter().map(record_to_wire));
            answers.push(record_to_wire(&soa));

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

        let mut lookup_options = LookupOptions::default();
        lookup_options.dnssec_ok = query
            .edns
            .as_ref()
            .is_some_and(|edns| edns.flags & 0x8000 != 0);

        let record_type = RecordType::from(question.qtype);
        let control_flow = if record_type == RecordType::SOA {
            handler
                .lookup(handler.origin(), record_type, None, lookup_options)
                .await
        } else {
            handler
                .lookup(&query_name, record_type, None, lookup_options)
                .await
        };

        build_auth_response(
            control_flow.map_result(),
            query,
            self.config.recursion_enabled,
            self.config.max_edns_payload_size,
        )
    }
}
