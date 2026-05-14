use crate::dns::Record;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub name: String,
    pub records: Vec<Record>,
}

#[allow(dead_code)]
impl Zone {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn remove_record(&mut self, name: &str, rtype: &str) -> bool {
        let before = self.records.len();
        self.records
            .retain(|r| !(r.name == name && r.rtype == rtype));
        before != self.records.len()
    }

    pub fn find(&self, name: &str, rtype: Option<&str>) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.name == name && rtype.is_none_or(|t| r.rtype == t))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_add_find_and_remove_records() {
        let mut zone = Zone::new("example.com");
        zone.add_record(Record::new("www.example.com", "A", 300, "1.2.3.4"));
        zone.add_record(Record::new("www.example.com", "AAAA", 300, "2001:db8::1"));

        assert_eq!(zone.find("www.example.com", None).len(), 2);
        assert_eq!(zone.find("www.example.com", Some("A")).len(), 1);
        assert!(zone.remove_record("www.example.com", "A"));
        assert_eq!(zone.find("www.example.com", None).len(), 1);
    }
}
