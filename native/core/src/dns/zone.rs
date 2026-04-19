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

    pub fn find(&self, name: &str, rtype: Option<&str>) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.name == name && rtype.is_none_or(|t| r.rtype == t))
            .cloned()
            .collect()
    }
}
