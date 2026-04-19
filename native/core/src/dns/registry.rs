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
