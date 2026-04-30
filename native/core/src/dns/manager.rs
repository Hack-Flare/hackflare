use crate::dns::Zone;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::collections::HashMap;
use std::fs;
use std::io;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsManager {
    zones: HashMap<String, Zone>,
}

impl Default for DnsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DnsManager {
    fn normalize_name(name: &str) -> String {
        name.trim().trim_end_matches('.').to_ascii_lowercase()
    }

    fn normalize_rtype(rtype: &str) -> String {
        rtype.trim().to_ascii_uppercase()
    }

    fn normalize_record_name_for_zone(zone_name: &str, record_name: &str) -> String {
        let normalized_zone = Self::normalize_name(zone_name);
        let normalized_record = Self::normalize_name(record_name);

        if normalized_record.is_empty() || normalized_record == "@" {
            return normalized_zone;
        }

        if normalized_record.ends_with(&format!(".{}", normalized_zone))
            || normalized_record == normalized_zone
        {
            return normalized_record;
        }

        format!("{}.{}", normalized_record, normalized_zone)
    }

    pub fn new() -> Self {
        Self {
            zones: HashMap::new(),
        }
    }

    pub fn create_zone(&mut self, name: impl Into<String>) -> &Zone {
        let name = Self::normalize_name(&name.into());
        self.zones
            .entry(name.clone())
            .or_insert_with(|| Zone::new(name.clone()));
        self.zones.get(&name).expect("just inserted")
    }

    pub fn delete_zone(&mut self, name: &str) -> bool {
        let normalized = Self::normalize_name(name);
        self.zones.remove(&normalized).is_some()
    }

    pub fn get_zone(&self, name: &str) -> Option<&Zone> {
        let normalized = Self::normalize_name(name);
        self.zones.get(&normalized)
    }

    pub fn get_zone_mut(&mut self, name: &str) -> Option<&mut Zone> {
        let normalized = Self::normalize_name(name);
        self.zones.get_mut(&normalized)
    }

    pub fn list_zones(&self) -> Vec<String> {
        self.zones.keys().cloned().collect()
    }

    pub fn add_record(
        &mut self,
        zone_name: &str,
        name: &str,
        rtype: &str,
        ttl: u32,
        data: &str,
    ) -> bool {
        let normalized_zone = Self::normalize_name(zone_name);
        let fqdn = Self::normalize_record_name_for_zone(&normalized_zone, name);
        let normalized_rtype = Self::normalize_rtype(rtype);

        if let Some(zone) = self.zones.get_mut(&normalized_zone) {
            zone.add_record(crate::dns::Record::new(
                fqdn,
                normalized_rtype,
                ttl,
                data.trim(),
            ));
            true
        } else {
            false
        }
    }

    pub fn remove_record(&mut self, zone_name: &str, name: &str, rtype: &str) -> bool {
        let normalized_zone = Self::normalize_name(zone_name);
        let fqdn = Self::normalize_record_name_for_zone(&normalized_zone, name);
        let normalized_rtype = Self::normalize_rtype(rtype);

        if let Some(zone) = self.zones.get_mut(&normalized_zone) {
            zone.remove_record(&fqdn, &normalized_rtype)
        } else {
            false
        }
    }

    pub fn find_records(&self, fqdn: &str, rtype: Option<&str>) -> Vec<crate::dns::Record> {
        let normalized_name = Self::normalize_name(fqdn);
        let normalized_rtype = rtype.map(Self::normalize_rtype);
        let mut out = Vec::new();
        for zone in self.zones.values() {
            for r in &zone.records {
                if Self::normalize_name(&r.name) == normalized_name
                    && normalized_rtype
                        .as_deref()
                        .is_none_or(|t| r.rtype.eq_ignore_ascii_case(t))
                {
                    out.push(r.clone());
                }
            }
        }
        out
    }

    pub fn find_answer_records(&self, fqdn: &str, qtype: Option<&str>) -> Vec<crate::dns::Record> {
        let Some(qtype) = qtype else {
            return self.find_records(fqdn, None);
        };

        let qtype = Self::normalize_rtype(qtype);
        if qtype == "CNAME" {
            return self.find_records(fqdn, Some("CNAME"));
        }

        let mut answer_chain = Vec::new();
        let mut current_name = Self::normalize_name(fqdn);
        let mut seen_names: HashSet<String> = HashSet::new();

        for _ in 0..8 {
            if !seen_names.insert(current_name.clone()) {
                break;
            }

            let direct = self.find_records(&current_name, Some(&qtype));
            if !direct.is_empty() {
                answer_chain.extend(direct);
                return answer_chain;
            }

            let cnames = self.find_records(&current_name, Some("CNAME"));
            if cnames.is_empty() {
                break;
            }

            let next_target = Self::normalize_name(&cnames[0].data);
            answer_chain.extend(cnames);
            if next_target.is_empty() {
                break;
            }
            current_name = next_target;
        }

        answer_chain
    }

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self).map_err(io::Error::other)?;
        fs::write(path, json)
    }

    pub fn load_from_file(path: &str) -> io::Result<Self> {
        let data = fs::read_to_string(path)?;
        let manager: DnsManager = serde_json::from_str(&data).map_err(io::Error::other)?;
        Ok(manager)
    }
}

#[cfg(test)]
mod tests {
    use super::DnsManager;

    #[test]
    fn add_record_normalizes_zone_and_relative_name() {
        let mut manager = DnsManager::new();
        manager.create_zone("Example.COM.");

        let added = manager.add_record("example.com", "www", "a", 300, "1.2.3.4");
        assert!(added);

        let recs = manager.find_records("WWW.Example.Com.", Some("A"));
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].name, "www.example.com");
        assert_eq!(recs[0].rtype, "A");
    }

    #[test]
    fn remove_record_handles_case_and_trailing_dot() {
        let mut manager = DnsManager::new();
        manager.create_zone("example.com");
        assert!(manager.add_record("example.com", "@", "TXT", 60, "hello"));

        let removed = manager.remove_record("EXAMPLE.COM.", "example.com.", "txt");
        assert!(removed);
        assert!(manager.find_records("example.com", Some("TXT")).is_empty());
    }

    #[test]
    fn find_answer_records_follows_local_cname_chain() {
        let mut manager = DnsManager::new();
        manager.create_zone("example.com");

        assert!(manager.add_record(
            "example.com",
            "www",
            "CNAME",
            300,
            "origin.example.com"
        ));
        assert!(manager.add_record(
            "example.com",
            "origin",
            "A",
            300,
            "203.0.113.10"
        ));

        let answers = manager.find_answer_records("www.example.com", Some("A"));
        assert_eq!(answers.len(), 2);
        assert_eq!(answers[0].rtype, "CNAME");
        assert_eq!(answers[1].rtype, "A");
        assert_eq!(answers[1].name, "origin.example.com");
    }
}
