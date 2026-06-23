use sqlx::PgPool;
use uuid::Uuid;

use crate::models::db::UserNotification;

pub(crate) async fn create_notification(
    db: &PgPool,
    user_id: &str,
    title: &str,
    message: &str,
    notif_type: &str,
    link: Option<&str>,
) -> Result<UserNotification, sqlx::Error> {
    sqlx::query_as::<_, UserNotification>(
        r#"
        INSERT INTO user_notifications (user_id, title, message, type, link)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(title)
    .bind(message)
    .bind(notif_type)
    .bind(link)
    .fetch_one(db)
    .await
}

pub(crate) async fn list_notifications(
    db: &PgPool,
    user_id: &str,
    limit: i64,
) -> Result<Vec<UserNotification>, sqlx::Error> {
    sqlx::query_as::<_, UserNotification>(
        r#"
        SELECT * FROM user_notifications
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(db)
    .await
}

pub(crate) async fn unread_count(db: &PgPool, user_id: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM user_notifications
        WHERE user_id = $1 AND NOT read
        "#,
    )
    .bind(user_id)
    .fetch_one(db)
    .await
}

pub(crate) async fn mark_read(db: &PgPool, id: Uuid, user_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE user_notifications SET read = TRUE
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(user_id)
    .execute(db)
    .await?;
    Ok(())
}

pub(crate) async fn mark_all_read(db: &PgPool, user_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE user_notifications SET read = TRUE
        WHERE user_id = $1 AND NOT read
        "#,
    )
    .bind(user_id)
    .execute(db)
    .await?;
    Ok(())
}
