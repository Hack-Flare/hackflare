use std::{sync::Arc, time::Duration};

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub(crate) config: Arc<Config>,
    pub(crate) http_client: reqwest::Client,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to create http client");

        Self {
            config: Arc::new(config),
            http_client,
        }
    }
}
