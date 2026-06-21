use axum::{
    Json, Router,
    extract::{Extension, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    models::CurrentUser,
    state::AppState,
};

#[derive(Serialize)]
pub(super) struct TrafficSummaryResponse {
    total_requests: i64,
    avg_processing_ms: f64,
    success_rate: f64,
    error_rate: f64,
}

#[derive(Serialize)]
pub(super) struct TimeseriesPoint {
    date: String,
    requests: i64,
    errors: i64,
    nxdomain: i64,
}

#[derive(Serialize)]
pub(super) struct ZoneTraffic {
    zone: String,
    requests: i64,
    errors: i64,
    avg_ms: f64,
}

#[derive(Serialize)]
pub(super) struct TopQuery {
    query: String,
    count: i64,
}

#[derive(Deserialize)]
pub(super) struct TimeseriesParams {
    days: Option<i32>,
}

#[derive(Deserialize)]
pub(super) struct TopQueriesParams {
    limit: Option<i32>,
}

async fn get_traffic_summary(
    db: &PgPool,
    user_id: Option<&str>,
) -> Result<TrafficSummaryResponse, StatusCode> {
    let (total_requests, avg_processing_us, success_count, error_count) =
        if let Some(uid) = user_id {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM dns_query_logs dql JOIN dns_zones dz ON dql.zone_name = dz.name WHERE dz.user_id = $1",
            )
            .bind(uid)
            .fetch_one(db)
            .await
            .map_err(|e| { tracing::error!("failed to fetch total_requests: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

            let avg: Option<f64> = sqlx::query_scalar(
                "SELECT AVG(processing_us)::float8 FROM dns_query_logs dql JOIN dns_zones dz ON dql.zone_name = dz.name WHERE dz.user_id = $1",
            )
            .bind(uid)
            .fetch_one(db)
            .await
            .map_err(|e| { tracing::error!("failed to fetch avg_processing_us: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

            let success: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM dns_query_logs dql JOIN dns_zones dz ON dql.zone_name = dz.name WHERE dql.response_code = 'NOERROR' AND dz.user_id = $1",
            )
            .bind(uid)
            .fetch_one(db)
            .await
            .map_err(|e| { tracing::error!("failed to fetch success_count: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

            let errors: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM dns_query_logs dql JOIN dns_zones dz ON dql.zone_name = dz.name WHERE dql.response_code NOT IN ('NOERROR', 'NXDOMAIN') AND dz.user_id = $1",
            )
            .bind(uid)
            .fetch_one(db)
            .await
            .map_err(|e| { tracing::error!("failed to fetch error_count: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

            (total, avg, success, errors)
        } else {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dns_query_logs")
                .fetch_one(db)
                .await
                .map_err(|e| { tracing::error!("failed to fetch total_requests: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

            let avg: Option<f64> = sqlx::query_scalar("SELECT AVG(processing_us)::float8 FROM dns_query_logs")
                .fetch_one(db)
                .await
                .map_err(|e| { tracing::error!("failed to fetch avg_processing_us: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

            let success: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dns_query_logs WHERE response_code = 'NOERROR'")
                .fetch_one(db)
                .await
                .map_err(|e| { tracing::error!("failed to fetch success_count: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

            let errors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dns_query_logs WHERE response_code NOT IN ('NOERROR', 'NXDOMAIN')")
                .fetch_one(db)
                .await
                .map_err(|e| { tracing::error!("failed to fetch error_count: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

            (total, avg, success, errors)
        };

    let total = total_requests.max(1);
    Ok(TrafficSummaryResponse {
        total_requests,
        avg_processing_ms: avg_processing_us.map(|u| u / 1000.0).unwrap_or(0.0),
        success_rate: (success_count as f64 / total as f64) * 100.0,
        error_rate: (error_count as f64 / total as f64) * 100.0,
    })
}

async fn get_timeseries(
    db: &PgPool,
    user_id: Option<&str>,
    days: i32,
) -> Result<Vec<TimeseriesPoint>, StatusCode> {
    let rows = if let Some(uid) = user_id {
        sqlx::query_as::<_, (chrono::NaiveDate, i64, i64, i64)>(
            r#"
            SELECT
                date_trunc('day', dql.timestamp)::date AS date,
                COUNT(*) AS requests,
                COUNT(*) FILTER (WHERE dql.response_code NOT IN ('NOERROR', 'NXDOMAIN')) AS errors,
                COUNT(*) FILTER (WHERE dql.response_code = 'NXDOMAIN') AS nxdomain
            FROM dns_query_logs dql
            JOIN dns_zones dz ON dql.zone_name = dz.name
            WHERE dz.user_id = $1 AND dql.timestamp >= now() - make_interval(days => $2::int)
            GROUP BY date_trunc('day', dql.timestamp)::date
            ORDER BY date ASC
            "#,
        )
        .bind(uid)
        .bind(days)
        .fetch_all(db)
        .await
        .map_err(|e| { tracing::error!("failed to fetch timeseries: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?
    } else {
        sqlx::query_as::<_, (chrono::NaiveDate, i64, i64, i64)>(
            r#"
            SELECT
                date_trunc('day', timestamp)::date AS date,
                COUNT(*) AS requests,
                COUNT(*) FILTER (WHERE response_code NOT IN ('NOERROR', 'NXDOMAIN')) AS errors,
                COUNT(*) FILTER (WHERE response_code = 'NXDOMAIN') AS nxdomain
            FROM dns_query_logs
            WHERE timestamp >= now() - make_interval(days => $1::int)
            GROUP BY date_trunc('day', timestamp)::date
            ORDER BY date ASC
            "#,
        )
        .bind(days)
        .fetch_all(db)
        .await
        .map_err(|e| { tracing::error!("failed to fetch timeseries: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?
    };

    Ok(rows
        .into_iter()
        .map(|(date, requests, errors, nxdomain)| TimeseriesPoint {
            date: date.format("%Y-%m-%d").to_string(),
            requests,
            errors,
            nxdomain,
        })
        .collect())
}

async fn get_by_zone(
    db: &PgPool,
    user_id: &str,
) -> Result<Vec<ZoneTraffic>, StatusCode> {
    let rows = sqlx::query_as::<_, (String, i64, i64, Option<f64>)>(
        r#"
        SELECT
            dql.zone_name,
            COUNT(*) AS requests,
            COUNT(*) FILTER (WHERE dql.response_code NOT IN ('NOERROR', 'NXDOMAIN')) AS errors,
            AVG(dql.processing_us)::float8 AS avg_ms
        FROM dns_query_logs dql
        JOIN dns_zones dz ON dql.zone_name = dz.name
        WHERE dz.user_id = $1
        GROUP BY dql.zone_name
        ORDER BY requests DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
    .map_err(|e| {
        tracing::error!("failed to fetch by-zone traffic: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(rows
        .into_iter()
        .map(|(zone, requests, errors, avg_us)| ZoneTraffic {
            zone,
            requests,
            errors,
            avg_ms: avg_us.map(|u| u / 1000.0).unwrap_or(0.0),
        })
        .collect())
}

async fn get_top_queries(
    db: &PgPool,
    user_id: Option<&str>,
    limit: i32,
) -> Result<Vec<TopQuery>, StatusCode> {
    let rows = if let Some(uid) = user_id {
        sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT dql.query_name, COUNT(*) AS count
            FROM dns_query_logs dql
            JOIN dns_zones dz ON dql.zone_name = dz.name
            WHERE dz.user_id = $1
            GROUP BY dql.query_name
            ORDER BY count DESC
            LIMIT $2
            "#,
        )
        .bind(uid)
        .bind(limit)
        .fetch_all(db)
        .await
        .map_err(|e| { tracing::error!("failed to fetch top queries: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?
    } else {
        sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT query_name, COUNT(*) AS count
            FROM dns_query_logs
            GROUP BY query_name
            ORDER BY count DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(db)
        .await
        .map_err(|e| { tracing::error!("failed to fetch top queries: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?
    };

    Ok(rows
        .into_iter()
        .map(|(query, count)| TopQuery { query, count })
        .collect())
}

// ── User-scoped endpoints ──

pub(super) async fn user_summary(
    State(db): State<PgPool>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<TrafficSummaryResponse>, StatusCode> {
    get_traffic_summary(&db, Some(&current_user.user.id)).await.map(Json)
}

pub(super) async fn user_timeseries(
    State(db): State<PgPool>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<TimeseriesParams>,
) -> Result<Json<Vec<TimeseriesPoint>>, StatusCode> {
    let days = params.days.unwrap_or(30).max(1).min(365);
    get_timeseries(&db, Some(&current_user.user.id), days).await.map(Json)
}

pub(super) async fn user_by_zone(
    State(db): State<PgPool>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ZoneTraffic>>, StatusCode> {
    get_by_zone(&db, &current_user.user.id).await.map(Json)
}

pub(super) async fn user_top_queries(
    State(db): State<PgPool>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<TopQueriesParams>,
) -> Result<Json<Vec<TopQuery>>, StatusCode> {
    let limit = params.limit.unwrap_or(10).max(1).min(100);
    get_top_queries(&db, Some(&current_user.user.id), limit).await.map(Json)
}

// ── Admin (platform-wide) endpoints ──

pub(super) async fn admin_summary(
    State(db): State<PgPool>,
) -> Result<Json<TrafficSummaryResponse>, StatusCode> {
    get_traffic_summary(&db, None).await.map(Json)
}

pub(super) async fn admin_timeseries(
    State(db): State<PgPool>,
    Query(params): Query<TimeseriesParams>,
) -> Result<Json<Vec<TimeseriesPoint>>, StatusCode> {
    let days = params.days.unwrap_or(30).max(1).min(365);
    get_timeseries(&db, None, days).await.map(Json)
}

pub(super) async fn admin_top_queries(
    State(db): State<PgPool>,
    Query(params): Query<TopQueriesParams>,
) -> Result<Json<Vec<TopQuery>>, StatusCode> {
    let limit = params.limit.unwrap_or(10).max(1).min(100);
    get_top_queries(&db, None, limit).await.map(Json)
}

// ── Routes ──

pub(super) fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/summary", get(user_summary))
        .route("/timeseries", get(user_timeseries))
        .route("/by-zone", get(user_by_zone))
        .route("/top-queries", get(user_top_queries))
}

pub(super) fn admin_traffic_routes() -> Router<AppState> {
    Router::new()
        .route("/traffic/summary", get(admin_summary))
        .route("/traffic/timeseries", get(admin_timeseries))
        .route("/traffic/top-queries", get(admin_top_queries))
}
