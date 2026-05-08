use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct Zone {
    pub id: u64,
    pub name: String,
    pub records: Vec<DnsRecord>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DnsRecord {
    pub name: String,
    pub record_type: RecordType,
    pub value: String,
    pub ttl: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RecordType {
    A,
    Aaaa,
    Cname,
    Txt,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewRecordInput {
    pub name: String,
    pub record_type: RecordType,
    pub value: String,
    pub ttl: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResolvedRecord {
    pub zone: String,
    pub name: String,
    pub record_type: RecordType,
    pub value: String,
    pub ttl: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DnsError {
    InvalidZoneName,
    ZoneAlreadyExists,
    ZoneNotFound,
    InvalidRecordName,
    InvalidRecordValue,
    InvalidRecordTtl,
}

pub struct DnsService {
    next_id: AtomicU64,
    zones: RwLock<HashMap<String, Zone>>,
}

impl DnsService {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            zones: RwLock::new(HashMap::new()),
        }
    }

    pub fn list_zones(&self) -> Vec<Zone> {
        let zones = self.zones.read().expect("zones read lock poisoned");
        let mut values = zones.values().cloned().collect::<Vec<_>>();
        values.sort_by(|a, b| a.name.cmp(&b.name));
        values
    }

    pub fn create_zone(&self, name: &str) -> Result<Zone, DnsError> {
        let normalized = normalize_zone_name(name).ok_or(DnsError::InvalidZoneName)?;
        let mut zones = self.zones.write().expect("zones write lock poisoned");

        if zones.contains_key(&normalized) {
            return Err(DnsError::ZoneAlreadyExists);
        }

        let zone = Zone {
            id: self.next_id.fetch_add(1, Ordering::Relaxed),
            name: normalized.clone(),
            records: Vec::new(),
        };

        zones.insert(normalized, zone.clone());
        Ok(zone)
    }

    pub fn add_record(&self, zone_name: &str, input: NewRecordInput) -> Result<Zone, DnsError> {
        let normalized_zone = normalize_zone_name(zone_name).ok_or(DnsError::InvalidZoneName)?;
        let mut zones = self.zones.write().expect("zones write lock poisoned");
        let zone = zones
            .get_mut(&normalized_zone)
            .ok_or(DnsError::ZoneNotFound)?;

        if input.ttl == 0 {
            return Err(DnsError::InvalidRecordTtl);
        }

        let normalized_name = normalize_record_name(&input.name, &normalized_zone)
            .ok_or(DnsError::InvalidRecordName)?;

        if !is_record_value_valid(input.record_type.clone(), &input.value) {
            return Err(DnsError::InvalidRecordValue);
        }

        zone.records.push(DnsRecord {
            name: normalized_name,
            record_type: input.record_type,
            value: input.value.trim().to_string(),
            ttl: input.ttl,
        });

        Ok(zone.clone())
    }

    pub fn find_records(&self, name: &str, record_type: Option<RecordType>) -> Vec<ResolvedRecord> {
        let normalized_name = match normalize_zone_name(name) {
            Some(value) => value,
            None => return Vec::new(),
        };

        let zones = self.zones.read().expect("zones read lock poisoned");
        let mut output = Vec::new();

        for zone in zones.values() {
            for record in &zone.records {
                if record.name != normalized_name {
                    continue;
                }

                if let Some(expected_type) = &record_type {
                    if &record.record_type != expected_type {
                        continue;
                    }
                }

                output.push(ResolvedRecord {
                    zone: zone.name.clone(),
                    name: record.name.clone(),
                    record_type: record.record_type.clone(),
                    value: record.value.clone(),
                    ttl: record.ttl,
                });
            }
        }

        output
    }
}

fn normalize_zone_name(input: &str) -> Option<String> {
    let value = input.trim().trim_end_matches('.').to_ascii_lowercase();

    if value.is_empty() || value.len() > 253 {
        return None;
    }

    let labels = value.split('.').collect::<Vec<_>>();
    if labels.len() < 2 {
        return None;
    }

    for label in labels {
        if label.is_empty() || label.len() > 63 {
            return None;
        }

        let first = label.chars().next()?;
        let last = label.chars().last()?;
        if !first.is_ascii_alphanumeric() || !last.is_ascii_alphanumeric() {
            return None;
        }

        if !label
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        {
            return None;
        }
    }

    Some(value)
}

fn normalize_record_name(input: &str, zone_name: &str) -> Option<String> {
    let value = input.trim().trim_end_matches('.').to_ascii_lowercase();

    if value == "@" {
        return Some(zone_name.to_string());
    }

    if value.contains('.') {
        let fqdn = normalize_zone_name(&value)?;
        if fqdn == zone_name || fqdn.ends_with(&format!(".{zone_name}")) {
            return Some(fqdn);
        }
        return None;
    }

    let candidate = format!("{value}.{zone_name}");
    normalize_zone_name(&candidate)
}

fn is_record_value_valid(record_type: RecordType, value: &str) -> bool {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return false;
    }

    match record_type {
        RecordType::A => trimmed
            .split('.')
            .map(|part| part.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()
            .map(|parts| parts.len() == 4)
            .unwrap_or(false),
        RecordType::Aaaa => trimmed.parse::<std::net::Ipv6Addr>().is_ok(),
        RecordType::Cname => normalize_zone_name(trimmed).is_some(),
        RecordType::Txt => trimmed.len() <= 512,
    }
}

#[cfg(test)]
mod tests {
    use super::{DnsError, DnsService, NewRecordInput, RecordType};

    #[test]
    fn creates_valid_zone() {
        let service = DnsService::new();

        let zone = service
            .create_zone("Example.COM")
            .expect("zone should be created");

        assert_eq!(zone.name, "example.com");
    }

    #[test]
    fn rejects_invalid_zone() {
        let service = DnsService::new();

        let result = service.create_zone("invalid_zone");

        assert!(matches!(result, Err(DnsError::InvalidZoneName)));
    }

    #[test]
    fn adds_record_to_zone() {
        let service = DnsService::new();
        service
            .create_zone("example.com")
            .expect("zone should be created");

        let zone = service
            .add_record(
                "example.com",
                NewRecordInput {
                    name: "www".to_string(),
                    record_type: RecordType::A,
                    value: "1.1.1.1".to_string(),
                    ttl: 60,
                },
            )
            .expect("record should be added");

        assert_eq!(zone.records.len(), 1);
        assert_eq!(zone.records[0].name, "www.example.com");
    }

    #[test]
    fn resolves_records_by_name_and_type() {
        let service = DnsService::new();
        service
            .create_zone("example.com")
            .expect("zone should be created");

        service
            .add_record(
                "example.com",
                NewRecordInput {
                    name: "@".to_string(),
                    record_type: RecordType::Txt,
                    value: "hello".to_string(),
                    ttl: 120,
                },
            )
            .expect("txt record should be added");

        service
            .add_record(
                "example.com",
                NewRecordInput {
                    name: "@".to_string(),
                    record_type: RecordType::A,
                    value: "8.8.8.8".to_string(),
                    ttl: 120,
                },
            )
            .expect("a record should be added");

        let txt_records = service.find_records("example.com", Some(RecordType::Txt));
        let a_records = service.find_records("example.com", Some(RecordType::A));

        assert_eq!(txt_records.len(), 1);
        assert_eq!(a_records.len(), 1);
    }
}
