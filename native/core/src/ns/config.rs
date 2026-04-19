use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NsConfig {
    pub bind_addr: String,
    pub port: u16,
    pub zone_file: Option<String>,
}

impl Default for NsConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0".to_string(),
            port: 53,
            zone_file: None,
        }
    }
}
