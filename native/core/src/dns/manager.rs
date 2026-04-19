use crate::dns::Zone;
use serde::{Deserialize, Serialize};
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
    pub fn new() -> Self {
        Self {
            zones: HashMap::new(),
        }
    }

    pub fn create_zone(&mut self, name: impl Into<String>) -> &Zone {
        let name = name.into();
        self.zones
            .entry(name.clone())
            .or_insert_with(|| Zone::new(name.clone()));
        self.zones.get(&name).expect("just inserted")
    }

    pub fn delete_zone(&mut self, name: &str) -> bool {
        self.zones.remove(name).is_some()
    }

    pub fn get_zone(&self, name: &str) -> Option<&Zone> {
        self.zones.get(name)
    }

    pub fn get_zone_mut(&mut self, name: &str) -> Option<&mut Zone> {
        self.zones.get_mut(name)
    }

    pub fn list_zones(&self) -> Vec<String> {
        self.zones.keys().cloned().collect()
    }

    pub fn find_records(&self, fqdn: &str, rtype: Option<&str>) -> Vec<crate::dns::Record> {
        let mut out = Vec::new();
        for zone in self.zones.values() {
            for r in &zone.records {
                if r.name == fqdn && rtype.is_none_or(|t| r.rtype.eq_ignore_ascii_case(t)) {
                    out.push(r.clone());
                }
            }
        }
        out
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
