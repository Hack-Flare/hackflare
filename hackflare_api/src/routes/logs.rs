use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    middleware,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    middlewares::auth_middleware,
    models::CurrentUser,
    state::AppState,
};

#[derive(Serialize)]
pub(super) struct LogEntryResponse {
    id: i64,
    timestamp: String,
    level: String,
    path: String,
    zone: String,
    status: i32,
    ms: i64,
}

#[derive(Serialize)]
pub(super) struct LogsSummaryResponse {
    errors_today: i64,
    warnings_today: i64,
    info_today: i64,
}

#[derive(Serialize)]
pub(super) struct LogsResponse {
    logs: Vec<LogEntryResponse>,
    summary: LogsSummaryResponse,
}

#[derive(Deserialize)]
pub(super) struct LogsQueryParams {
    zone: Option<String>,
}

fn derive_level(response_code: &str) -> &'static str {
    match response_code {
        "NOERROR" => "info",
        "NXDOMAIN" => "warning",
        _ => "error",
    }
}

fn derive_status(response_code: &str) -> i32 {
    match response_code {
        "NOERROR" => 0,
        "FORMERR" => 1,
        "SERVFAIL" => 2,
        "NXDOMAIN" => 3,
        "NOTIMP" => 4,
        "REFUSED" => 5,
        "YXDOMAIN" => 6,
        "XRRSET" => 7,
        "NOTAUTH" => 9,
        _ => 0,
    }
}

pub(super) async fn list_query_logs(
    State(db): State<PgPool>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<LogsQueryParams>,
) -> Result<Json<LogsResponse>, StatusCode> {
    let zone_filter = params.zone.unwrap_or_default();

    let rows = sqlx::query_as::<_, (i64, chrono::DateTime<chrono::Utc>, String, String, String, i32)>(
        r#"
        SELECT dql.id, dql.timestamp, dql.query_name, dql.response_code, dql.zone_name, dql.processing_us
        FROM dns_query_logs dql
        JOIN dns_zones dz ON dql.zone_name = dz.name
        WHERE dz.user_id = $1
          AND dql.timestamp >= now() - interval '24 hours'
          AND ($2 = '' OR dql.zone_name = $2)
        ORDER BY dql.id DESC
        LIMIT 200
        "#,
    )
    .bind(&current_user.user.id)
    .bind(&zone_filter)
    .fetch_all(&db)
    .await
    .map_err(|e| {
        tracing::error!("failed to fetch query logs: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let summary = sqlx::query_as::<_, (i64, i64, i64)>(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE dql.response_code NOT IN ('NOERROR', 'NXDOMAIN') AND dql.timestamp >= now()::date) AS errors_today,
            COUNT(*) FILTER (WHERE dql.response_code = 'NXDOMAIN' AND dql.timestamp >= now()::date) AS warnings_today,
            COUNT(*) FILTER (WHERE dql.response_code = 'NOERROR' AND dql.timestamp >= now()::date) AS info_today
        FROM dns_query_logs dql
        JOIN dns_zones dz ON dql.zone_name = dz.name
        WHERE dz.user_id = $1
          AND ($2 = '' OR dql.zone_name = $2)
        "#,
    )
    .bind(&current_user.user.id)
    .bind(&zone_filter)
    .fetch_one(&db)
    .await
    .map_err(|e| {
        tracing::error!("failed to fetch log summary: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let logs: Vec<LogEntryResponse> = rows
        .into_iter()
        .map(|(id, ts, query_name, response_code, zone_name, processing_us)| {
            LogEntryResponse {
                id,
                timestamp: ts.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                level: derive_level(&response_code).to_string(),
                path: query_name,
                zone: zone_name,
                status: derive_status(&response_code),
                ms: (processing_us as i64) / 1000,
            }
        })
        .collect();

    Ok(Json(LogsResponse {
        logs,
        summary: LogsSummaryResponse {
            errors_today: summary.0,
            warnings_today: summary.1,
            info_today: summary.2,
        },
    }))
}

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/query-logs", get(list_query_logs))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
}
