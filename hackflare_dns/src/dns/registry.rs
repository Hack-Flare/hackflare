use crate::dns::Record;
use std::collections::HashMap;

pub type Encoder = fn(&Record) -> Option<Vec<u8>>;

pub struct Registry {
    map: HashMap<String, Encoder>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, f: Encoder) {
        self.map.insert(name.to_uppercase(), f);
    }

    pub fn get(&self, name: &str) -> Option<&Encoder> {
        self.map.get(&name.to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dns::Record;

    fn encode_stub(_record: &Record) -> Option<Vec<u8>> {
        Some(vec![1, 2, 3])
    }

    #[test]
    fn registry_lookup_is_case_insensitive() {
        let mut registry = Registry::new();
        registry.register("a", encode_stub);

        assert!(registry.get("A").is_some());
        assert!(registry.get("a").is_some());
        assert!(registry.get("missing").is_none());
    }
}
