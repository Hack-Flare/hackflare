use std::sync::Arc;

use axum::{
    extract::{Extension, Request, State},
    middleware::Next,
    response::Response,
};
use reqwest::StatusCode;

use crate::{config::Config, models::CurrentUser};

pub(crate) async fn require_admin(
    State(config): State<Arc<Config>>,
    Extension(current_user): Extension<CurrentUser>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    let user_email = current_user.user.email.to_lowercase();

    if config.admin_emails.is_empty() {
        warn!("no admin emails configured, denying all");
        return Err((StatusCode::FORBIDDEN, "forbidden"));
    }

    if config.admin_emails.iter().any(|e| e == &user_email) {
        return Ok(next.run(req).await);
    }

    warn!(email = %user_email, "non-admin user attempted admin access");
    Err((StatusCode::FORBIDDEN, "forbidden"))
}
