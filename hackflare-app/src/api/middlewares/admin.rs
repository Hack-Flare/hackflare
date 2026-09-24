use axum::{
    extract::{Extension, Request, State},
    middleware::Next,
    response::Response,
};
use reqwest::StatusCode;

use crate::{models::CurrentUser, state::AppState};

pub(crate) async fn require_admin(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    let user_email = current_user.user.email.to_lowercase();

    // Check live overrides first (runtime config), fall back to static config.
    let admin_emails: Vec<String> = {
        let overrides = state.live_overrides.read().await;
        overrides
            .get("API_ADMIN_EMAILS")
            .map(|v| v.split(',').map(|s| s.trim().to_lowercase()).collect())
            .unwrap_or_else(|| state.config.admin_emails.clone())
    };

    if admin_emails.is_empty() {
        warn!("no admin emails configured, denying all");
        return Err((StatusCode::FORBIDDEN, "forbidden"));
    }

    if admin_emails.iter().any(|e| e == &user_email) {
        return Ok(next.run(req).await);
    }

    warn!(email = %user_email, "non-admin user attempted admin access");
    Err((StatusCode::FORBIDDEN, "forbidden"))
}
