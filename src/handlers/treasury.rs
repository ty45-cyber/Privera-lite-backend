use axum::{
    extract::{Extension, Path, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use reqwest::Client;
use crate::{
    AppState,
    errors::{AppError, AppResult},
    models::{user::AuthClaims, treasury::{CreateTreasuryRequest, ApprovalDecision}},
    services::{treasury, market},
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/treasury/requests", post(create).get(list_all))
        .route("/treasury/requests/pending", get(list_pending))
        .route("/treasury/requests/:id/approve", post(approve))
        .route("/treasury/requests/:id/reject", post(reject))
        .route("/treasury/requests/:id/risk-score", get(risk_score))
}

async fn create(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<CreateTreasuryRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if !["admin", "finance"].contains(&claims.role.as_str()) {
        return Err(AppError::Forbidden);
    }
    if req.required_approvals < 1 || req.required_approvals > 5 {
        return Err(AppError::Validation("required_approvals must be 1–5".into()));
    }
    let request = treasury::create_request(&state.pool, req, &claims.user_id).await?;
    Ok(Json(serde_json::json!({ "request": request })))
}

async fn list_pending(
    State(state): State<Arc<AppState>>,
    Extension(_): Extension<AuthClaims>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = treasury::list_pending(&state.pool).await?;
    Ok(Json(serde_json::json!({ "requests": rows })))
}

async fn list_all(
    State(state): State<Arc<AppState>>,
    Extension(_): Extension<AuthClaims>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = treasury::list_all(&state.pool).await?;
    Ok(Json(serde_json::json!({ "requests": rows })))
}

async fn approve(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    if !["admin", "finance"].contains(&claims.role.as_str()) {
        return Err(AppError::Forbidden);
    }
    let decision = ApprovalDecision {
        decision: "approved".into(),
        note: body.get("note").and_then(|v| v.as_str()).map(String::from),
    };
    let req = treasury::record_decision(&state.pool, &id, &claims.user_id, "Approver", decision).await?;
    Ok(Json(serde_json::json!({ "request": req })))
}

async fn reject(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    if !["admin", "finance"].contains(&claims.role.as_str()) {
        return Err(AppError::Forbidden);
    }
    let decision = ApprovalDecision {
        decision: "rejected".into(),
        note: body.get("note").and_then(|v| v.as_str()).map(String::from),
    };
    let req = treasury::record_decision(&state.pool, &id, &claims.user_id, "Approver", decision).await?;
    Ok(Json(serde_json::json!({ "request": req })))
}

async fn risk_score(
    State(state): State<Arc<AppState>>,
    Extension(_): Extension<AuthClaims>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let request = sqlx::query!(
        "SELECT amount FROM treasury_requests WHERE id = ?", id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Treasury request {id}")))?;

    let http = Client::new();
    let score = market::get_risk_score(&state.cfg, &http, &id, request.amount).await?;
    Ok(Json(serde_json::json!({ "risk": score })))
}