use crate::dns::Record;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub name: String,
    pub records: Vec<Record>,
}

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

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_add_and_remove_records() {
        let mut zone = Zone::new("example.com");
        zone.add_record(Record::new("www.example.com", "A", 300, "1.2.3.4"));
        zone.add_record(Record::new("www.example.com", "AAAA", 300, "2001:db8::1"));

        assert_eq!(zone.records.len(), 2);
        assert!(zone.remove_record("www.example.com", "A"));
        assert_eq!(zone.records.len(), 1);
    }
}
