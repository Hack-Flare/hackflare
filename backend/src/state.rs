use std::sync::Arc;

use hackclub_auth_api::HCAuth;

use crate::config::Config;
use crate::domain::auth::AuthService;
use crate::domain::dns::DnsService;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub auth: Arc<AuthService>,
    pub hackclub_auth: Option<Arc<HCAuth>>,
    pub dns: Arc<DnsService>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let hackclub_auth = if config.hackclub.is_enabled() {
            Some(Arc::new(HCAuth::new(
                config.hackclub.client_id.as_deref().expect("hackclub client id missing"),
                config
                    .hackclub
                    .client_secret
                    .as_deref()
                    .expect("hackclub client secret missing"),
                config
                    .hackclub
                    .redirect_uri
                    .as_deref()
                    .expect("hackclub redirect uri missing"),
            )))
        } else {
            None
        };

        Self {
            config: Arc::new(config),
            auth: Arc::new(AuthService::new()),
            hackclub_auth,
            dns: Arc::new(DnsService::new()),
        }
    }
}
