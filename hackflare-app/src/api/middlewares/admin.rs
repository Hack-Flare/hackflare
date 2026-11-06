use axum::{
    extract::{Extension, Request, State},
    middleware::Next,
    response::Response,
};
use reqwest::StatusCode;

use crate::{models::CurrentUser, state::AppState};

pub(crate) fn is_admin_user(user: &crate::api::models::db::User, config: &crate::config::Config) -> bool {
    if user.is_admin {
        return true;
    }
    let email = user.email.to_lowercase();
    config.admin_emails.iter().any(|e| e == &email)
}

pub(crate) async fn require_admin(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    if is_admin_user(&current_user.user, &state.config) {
        return Ok(next.run(req).await);
    }

    warn!(email = %current_user.user.email, "non-admin user attempted admin access");
    Err((StatusCode::FORBIDDEN, "forbidden"))
}
