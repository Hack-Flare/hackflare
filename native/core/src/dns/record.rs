use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Record {
    pub name: String,
    pub rtype: String,
    pub ttl: u32,
    pub data: String,
}

impl Record {
    pub fn new(
        name: impl Into<String>,
        rtype: impl Into<String>,
        ttl: u32,
        data: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            rtype: rtype.into(),
            ttl,
            data: data.into(),
        }
    }
}
