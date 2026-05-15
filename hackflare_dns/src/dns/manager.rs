use crate::dns::Zone;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsManager {
    zones: HashMap<String, Zone>,
    #[serde(skip)]
    records_by_name: HashMap<String, Vec<crate::dns::Record>>,
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

    pub fn new() -> Self {
        Self {
            zones: HashMap::new(),
            records_by_name: HashMap::new(),
        }
    }

    pub fn find_records(&self, fqdn: &str, rtype: Option<&str>) -> Vec<crate::dns::Record> {
        let normalized_name = Self::normalize_name(fqdn);
        let normalized_rtype = rtype.map(Self::normalize_rtype);
        let Some(records) = self.records_by_name.get(&normalized_name) else {
            return Vec::new();
        };

        records
            .iter()
            .filter(|record| {
                normalized_rtype
                    .as_deref()
                    .is_none_or(|rtype| record.rtype.eq_ignore_ascii_case(rtype))
            })
            .cloned()
            .collect()
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

    #[cfg(test)]
    pub(crate) fn insert_zone(&mut self, zone: Zone) {
        for record in &zone.records {
            self.records_by_name
                .entry(record.name.clone())
                .or_default()
                .push(record.clone());
        }
        self.zones.insert(zone.name.clone(), zone);
    }
}
