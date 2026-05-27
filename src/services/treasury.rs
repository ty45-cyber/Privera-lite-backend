use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::Utc;
use crate::{
    errors::{AppError, AppResult},
    models::treasury::*,
};

pub async fn create_request(
    pool: &MySqlPool,
    req: CreateTreasuryRequest,
    requested_by: &str,
) -> AppResult<TreasuryRequest> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    sqlx::query!(
        r#"INSERT INTO treasury_requests
           (id, title, amount, currency, purpose, risk_level, status, required_approvals, current_approvals, requested_by, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, 'pending', 'pending', ?, 0, ?, ?, ?)"#,
        id, req.title, req.amount, req.currency, req.purpose,
        req.required_approvals, requested_by, now, now
    )
    .execute(pool)
    .await?;

    Ok(TreasuryRequest {
        id, title: req.title, amount: req.amount, currency: req.currency,
        purpose: req.purpose, risk_level: "pending".into(),
        status: "pending".into(), required_approvals: req.required_approvals,
        current_approvals: 0, requested_by: requested_by.into(),
        created_at: now, updated_at: now,
    })
}

pub async fn list_pending(pool: &MySqlPool) -> AppResult<Vec<TreasuryRequest>> {
    let rows = sqlx::query_as!(TreasuryRequest,
        "SELECT id, title, amount, currency, purpose, risk_level, status, required_approvals, current_approvals, requested_by, created_at, updated_at FROM treasury_requests WHERE status = 'pending' ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_all(pool: &MySqlPool) -> AppResult<Vec<TreasuryRequest>> {
    let rows = sqlx::query_as!(TreasuryRequest,
        "SELECT id, title, amount, currency, purpose, risk_level, status, required_approvals, current_approvals, requested_by, created_at, updated_at FROM treasury_requests ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn record_decision(
    pool: &MySqlPool,
    request_id: &str,
    approver_id: &str,
    approver_name: &str,
    decision: ApprovalDecision,
) -> AppResult<TreasuryRequest> {
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM treasury_approvals WHERE request_id = ? AND approver_id = ?",
        request_id, approver_id
    )
    .fetch_one(pool)
    .await?;

    if existing > 0 {
        return Err(AppError::Conflict("Already voted on this request".into()));
    }

    let approval_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    sqlx::query!(
        r#"INSERT INTO treasury_approvals (id, request_id, approver_id, approver_name, decision, note, decided_at)
           VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        approval_id, request_id, approver_id, approver_name, decision.decision, decision.note, now
    )
    .execute(pool)
    .await?;

    if decision.decision == "approved" {
        sqlx::query!(
            r#"UPDATE treasury_requests
               SET current_approvals = current_approvals + 1,
                   status = CASE WHEN current_approvals + 1 >= required_approvals THEN 'approved' ELSE status END,
                   updated_at = ?
               WHERE id = ?"#,
            now, request_id
        )
        .execute(pool)
        .await?;
    } else {
        sqlx::query!(
            "UPDATE treasury_requests SET status = 'rejected', updated_at = ? WHERE id = ?",
            now, request_id
        )
        .execute(pool)
        .await?;
    }

    let updated = sqlx::query_as!(TreasuryRequest,
        "SELECT id, title, amount, currency, purpose, risk_level, status, required_approvals, current_approvals, requested_by, created_at, updated_at FROM treasury_requests WHERE id = ?",
        request_id
    )
    .fetch_one(pool)
    .await?;

    Ok(updated)
}
