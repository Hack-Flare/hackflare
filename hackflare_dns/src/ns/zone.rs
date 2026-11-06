//! In-memory zone storage.
//!
//! A [`Zone`] holds records grouped into resource record sets keyed by owner name and record type.
//! It provides the lookup behavior used by the authoritative server

use std::collections::HashMap;

use crate::dns::rr::{RData, Record, RecordType};

/// The records that share one owner name and record type.
#[derive(Debug, Clone)]
struct RecordSet {
    name: String,
    records: Vec<Record>,
}

impl RecordSet {
    fn new(name: String) -> Self {
        Self {
            name,
            records: Vec::new(),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn records(&self) -> impl Iterator<Item = &Record> {
        self.records.iter()
    }

    fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    fn first_mut(&mut self) -> Option<&mut Record> {
        self.records.first_mut()
    }

    fn add(&mut self, record: Record) {
        if let Some(existing) = self
            .records
            .iter_mut()
            .find(|existing| existing.data == record.data)
        {
            existing.ttl = record.ttl;
            return;
        }
        self.records.push(record);
    }

    fn remove_value(&mut self, data: &RData) -> bool {
        let before = self.records.len();
        self.records.retain(|record| &record.data != data);
        before != self.records.len()
    }
}

/// The result of a zone lookup.
#[derive(Debug, Default)]
pub struct Lookup {
    pub records: Vec<Record>,
    pub additionals: Vec<Record>,
    pub nx_domain: bool,
}

/// An in-memory authoritative zone.
#[derive(Debug, Clone)]
pub struct Zone {
    origin: String,
    serial: u32,
    records: HashMap<(String, RecordType), RecordSet>,
}

impl Zone {
    #[must_use]
    pub fn new(origin: impl Into<String>, serial: u32) -> Self {
        Self {
            origin: origin.into(),
            serial,
            records: HashMap::new(),
        }
    }

    #[must_use]
    pub fn origin(&self) -> &str {
        &self.origin
    }

    #[must_use]
    pub const fn serial(&self) -> u32 {
        self.serial
    }

    fn set_serial(&mut self, serial: u32) {
        self.serial = serial;
        let key = (self.origin.clone(), RecordType::Soa);
        let Some(record) = self.records.get_mut(&key).and_then(RecordSet::first_mut) else {
            return;
        };
        if let RData::Soa {
            serial: current, ..
        } = &mut record.data
        {
            *current = serial;
        }
    }

    /// Increment the serial, never dropping below `floor`.
    pub fn bump_serial(&mut self, floor: u32) -> u32 {
        let serial = self.serial.saturating_add(1).max(floor);
        self.set_serial(serial);
        serial
    }

    /// Insert or update a record. Returns `true` when the record was added or refreshed.
    pub fn upsert(&mut self, record: Record) -> bool {
        let key = (record.name.clone(), record.record_type());
        self.records
            .entry(key)
            .or_insert_with(|| RecordSet::new(record.name.clone()))
            .add(record);
        true
    }

    /// Remove every record with the given owner name and type.
    pub fn remove(&mut self, name: &str, rtype: RecordType) -> bool {
        self.records.remove(&(name.to_string(), rtype)).is_some()
    }

    /// Remove a single record value from a set.
    pub fn remove_value(&mut self, name: &str, rtype: RecordType, data: &RData) -> bool {
        let key = (name.to_string(), rtype);
        let Some(set) = self.records.get_mut(&key) else {
            return false;
        };
        if !set.remove_value(data) {
            return false;
        }
        if set.is_empty() {
            self.records.remove(&key);
        }
        true
    }

    pub fn all_records(&self) -> impl Iterator<Item = &Record> {
        self.records.values().flat_map(RecordSet::records)
    }

    /// Look up records for a query name and type.
    #[must_use]
    pub fn lookup(&self, name: &str, qtype: RecordType) -> Lookup {
        if qtype == RecordType::Soa {
            let records = self.find_records(name, RecordType::Soa);
            return Lookup {
                records,
                ..Lookup::default()
            };
        }

        let any_exact = self.has_exact_name(name);
        let any_wildcard = self.has_wildcard_name(name);

        let mut records = self.find_records(name, qtype);
        let mut additionals = Vec::new();

        if records.is_empty() && qtype == RecordType::Any {
            records = self.all_at(name);
            if records.is_empty() {
                records = self.all_wildcard_at(name);
            }
        }

        if records.is_empty() && qtype != RecordType::Cname {
            let cname = self.find_records(name, RecordType::Cname);
            if !cname.is_empty() {
                if qtype == RecordType::A || qtype == RecordType::Aaaa {
                    let target = record_target(&cname[0]);
                    if let Some(target) = target {
                        additionals = self.find_records(target, qtype);
                    }
                }
                records = cname;
            }
        }

        if records.is_empty() && matches!(qtype, RecordType::A | RecordType::Aaaa) {
            let aname = self.find_records(name, RecordType::Aname);
            if let Some(record) = aname.first() {
                if let Some(target) = record_target(record) {
                    records = self.find_records(target, qtype);
                }
                if records.is_empty() {
                    records = aname;
                }
            }
        }

        if records.is_empty() && qtype != RecordType::Ns {
            let delegation = self.find_delegation(name);
            if !delegation.is_empty() {
                records = delegation;
            }
        }

        if records.is_empty() && !any_exact && !any_wildcard {
            let delegation = self.find_delegation(name);
            if delegation.is_empty() {
                return Lookup {
                    nx_domain: true,
                    ..Lookup::default()
                };
            }
            records = delegation;
        }

        Lookup {
            records,
            additionals,
            nx_domain: false,
        }
    }

    fn has_exact_name(&self, name: &str) -> bool {
        let suffix = format!(".{name}");
        self.records
            .keys()
            .any(|(owner, _)| owner == name || owner.ends_with(&suffix))
    }

    fn has_wildcard_name(&self, name: &str) -> bool {
        self.ancestors(name).iter().any(|ancestor| {
            let wildcard = format!("*.{ancestor}.");
            self.records.keys().any(|(owner, _)| owner == &wildcard)
        })
    }

    fn find_records(&self, name: &str, rtype: RecordType) -> Vec<Record> {
        if let Some(set) = self.records.get(&(name.to_string(), rtype)) {
            return set.records.clone();
        }
        for ancestor in self.ancestors(name) {
            let wildcard = format!("*.{ancestor}.");
            if let Some(set) = self.records.get(&(wildcard, rtype)) {
                return set
                    .records
                    .iter()
                    .cloned()
                    .map(|mut record| {
                        record.name = name.to_string();
                        record
                    })
                    .collect();
            }
        }
        Vec::new()
    }

    fn all_at(&self, name: &str) -> Vec<Record> {
        self.records
            .values()
            .filter(|set| set.name() == name)
            .flat_map(|set| set.records.clone())
            .collect()
    }

    fn all_wildcard_at(&self, name: &str) -> Vec<Record> {
        for ancestor in self.ancestors(name) {
            let wildcard = format!("*.{ancestor}.");
            let records: Vec<Record> = self
                .records
                .values()
                .filter(|set| set.name() == wildcard)
                .flat_map(|set| set.records.clone())
                .map(|mut record| {
                    record.name = name.to_string();
                    record
                })
                .collect();
            if !records.is_empty() {
                return records;
            }
        }
        Vec::new()
    }

    fn find_delegation(&self, name: &str) -> Vec<Record> {
        let mut current = name.trim_end_matches('.');
        loop {
            let owner = format!("{current}.");
            if owner != self.origin
                && let Some(set) = self.records.get(&(owner, RecordType::Ns))
            {
                return set.records.clone();
            }
            let Some((_, suffix)) = current.split_once('.') else {
                return Vec::new();
            };
            if suffix.is_empty() {
                return Vec::new();
            }
            current = suffix;
        }
    }

    fn ancestors(&self, name: &str) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut current = name.trim_end_matches('.');
        while let Some((_, suffix)) = current.split_once('.') {
            if suffix.is_empty() {
                break;
            }
            ancestors.push(suffix.to_string());
            current = suffix;
        }
        ancestors
    }
}

fn record_target(record: &Record) -> Option<&str> {
    match &record.data {
        RData::Cname(target) | RData::Aname(target) => Some(target),
        _ => None,
    }
}
