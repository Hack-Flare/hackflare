use std::{sync::Arc, time::Duration};

use anyhow::Result;
use axum::extract::FromRef;
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::config::Config;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub(crate) config: Arc<Config>,
    pub(crate) http_client: reqwest::Client,
    pub(crate) db: PgPool,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to create http client");
        info!("http client initialized");

        let db = PgPoolOptions::new()
            .max_connections(50)
            .connect(config.database_url.as_str())
            .await?;
        info!("database connection pool initialized");

        Ok(Self {
            config: Arc::new(config),
            http_client,
            db,
        })
    }
}
