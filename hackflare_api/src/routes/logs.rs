use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    middleware,
    routing::get,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::{middlewares::auth_middleware, state::AppState};

#[derive(Serialize)]
pub(super) struct LogEntryResponse {
    id: i64,
    timestamp: String,
    level: String,
    path: String,
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
) -> Result<Json<LogsResponse>, StatusCode> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, i32, String, i32)>(
        r#"
        SELECT id, query_name, query_type, response_code, response_size, source_ip, processing_us
        FROM dns_query_logs
        WHERE created_at >= now() - interval '24 hours'
        ORDER BY id DESC
        LIMIT 200
        "#,
    )
    .fetch_all(&db)
    .await
    .map_err(|e| {
        tracing::error!("failed to fetch query logs: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let summary = sqlx::query_as::<_, (i64, i64, i64)>(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE response_code NOT IN ('NOERROR', 'NXDOMAIN') AND created_at >= now()::date) AS errors_today,
            COUNT(*) FILTER (WHERE response_code = 'NXDOMAIN' AND created_at >= now()::date) AS warnings_today,
            COUNT(*) FILTER (WHERE response_code = 'NOERROR' AND created_at >= now()::date) AS info_today
        FROM dns_query_logs
        "#,
    )
    .fetch_one(&db)
    .await
    .map_err(|e| {
        tracing::error!("failed to fetch log summary: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let logs: Vec<LogEntryResponse> = rows
        .into_iter()
        .map(|(id, query_name, _query_type, response_code, _resp_size, _src_ip, processing_us)| {
            LogEntryResponse {
                id,
                timestamp: String::new(),
                level: derive_level(&response_code).to_string(),
                path: query_name,
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
