use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::Utc;
use crate::{
    errors::{AppError, AppResult},
    models::audit::*,
};

pub async fn submit_request(
    pool: &MySqlPool,
    auditor_id: &str,
    auditor_name: &str,
    req: CreateAuditRequest,
) -> AppResult<AuditRequest> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    sqlx::query!(
        r#"INSERT INTO audit_requests (id, auditor_id, auditor_name, resource_type, resource_id, reason, status, created_at)
           VALUES (?, ?, ?, ?, ?, ?, 'pending', ?)"#,
        id, auditor_id, auditor_name, req.resource_type, req.resource_id, req.reason, now
    )
    .execute(pool)
    .await?;

    Ok(AuditRequest {
        id, auditor_id: auditor_id.into(), auditor_name: auditor_name.into(),
        resource_type: req.resource_type, resource_id: req.resource_id,
        reason: req.reason, status: "pending".into(), created_at: now,
    })
}

pub async fn grant_request(
    pool: &MySqlPool,
    request_id: &str,
    admin_id: &str,
    admin_role: &str,
    decision: GrantAuditRequest,
) -> AppResult<AuditRequest> {
    if !["granted", "denied"].contains(&decision.decision.as_str()) {
        return Err(AppError::Validation("Decision must be granted or denied".into()));
    }

    sqlx::query!(
        "UPDATE audit_requests SET status = ? WHERE id = ? AND status = 'pending'",
        decision.decision, request_id
    )
    .execute(pool)
    .await?;

    write_log(pool, admin_id, admin_role, "audit_decision", "audit_request", request_id, None).await?;

    let updated = sqlx::query_as!(AuditRequest,
        "SELECT id, auditor_id, auditor_name, resource_type, resource_id, reason, status, created_at FROM audit_requests WHERE id = ?",
        request_id
    )
    .fetch_one(pool)
    .await?;

    Ok(updated)
}

pub async fn write_log(
    pool: &MySqlPool,
    actor_id: &str,
    actor_role: &str,
    action: &str,
    resource_type: &str,
    resource_id: &str,
    metadata: Option<&str>,
) -> AppResult<()> {
    let id = Uuid::new_v4().to_string();
    sqlx::query!(
        r#"INSERT INTO audit_logs (id, actor_id, actor_role, action, resource_type, resource_id, metadata, occurred_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        id, actor_id, actor_role, action, resource_type, resource_id, metadata, Utc::now()
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_logs(pool: &MySqlPool) -> AppResult<Vec<AuditLog>> {
    let rows = sqlx::query_as!(AuditLog,
        "SELECT id, actor_id, actor_role, action, resource_type, resource_id, metadata, occurred_at FROM audit_logs ORDER BY occurred_at DESC LIMIT 500"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_requests(pool: &MySqlPool) -> AppResult<Vec<AuditRequest>> {
    let rows = sqlx::query_as!(AuditRequest,
        "SELECT id, auditor_id, auditor_name, resource_type, resource_id, reason, status, created_at FROM audit_requests ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
