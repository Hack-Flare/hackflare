use axum::{Json, extract::State};
use serde::Serialize;
use sqlx::PgPool;

use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
    pub dns_zones: usize,
}

pub(crate) async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let (database, dns_zones) = tokio::join!(
        check_db(&state.db),
        count_zones(state.dns_authority.clone()),
    );

    let all_ok = database == "ok";
    Json(HealthResponse {
        status: if all_ok {
            "ok".into()
        } else {
            "degraded".into()
        },
        database,
        dns_zones,
    })
}

async fn check_db(db: &PgPool) -> String {
    match sqlx::query("SELECT 1").execute(db).await {
        Ok(_) => "ok".into(),
        Err(e) => format!("error: {e}"),
    }
}

async fn count_zones(authority: std::sync::Arc<hackflare_dns::ns::AuthorityStore>) -> usize {
    authority.list_zones().await.len()
}
