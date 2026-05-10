use std::sync::Arc;

use crate::config::Config;
use crate::domain::auth::AuthService;
use crate::domain::dns::DnsService;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub auth: Arc<AuthService>,
    pub dns: Arc<DnsService>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            auth: Arc::new(AuthService::new()),
            dns: Arc::new(DnsService::new()),
        }
    }
}
