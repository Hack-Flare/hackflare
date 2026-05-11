use std::sync::Arc;

use hackclub_auth_api::HCAuth;

use crate::config::Config;
use crate::domain::auth::AuthService;
use crate::domain::dns::DnsService;
use crate::persistence::{BackendSnapshot, BackendStore};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub auth: Arc<AuthService>,
    pub hackclub_auth: Option<Arc<HCAuth>>,
    pub dns: Arc<DnsService>,
    persistence: Arc<BackendStore>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let persistence = Arc::new(BackendStore::new(config.database_url.clone()));
        let snapshot = persistence.load().expect("failed to load backend state from postgres");

        let hackclub_auth = if config.hackclub.is_enabled() {
            Some(Arc::new(HCAuth::new(
                config
                    .hackclub
                    .client_id
                    .as_deref()
                    .expect("hackclub client id missing"),
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

        let auth_snapshot = snapshot.as_ref().map(|state| state.auth.clone());
        let dns_snapshot = snapshot.as_ref().map(|state| state.dns.clone());

        Self {
            config: Arc::new(config),
            auth: Arc::new(AuthService::from_snapshot(auth_snapshot)),
            hackclub_auth,
            dns: Arc::new(DnsService::from_snapshot(dns_snapshot)),
            persistence,
        }
    }

    pub fn persist(&self) {
        let snapshot = BackendSnapshot {
            auth: self.auth.snapshot(),
            dns: self.dns.snapshot(),
        };

        self.persistence
            .save(&snapshot)
            .expect("failed to persist backend state to postgres");
    }
}
