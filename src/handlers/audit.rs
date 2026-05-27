use axum::{
    extract::{Extension, Path, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use crate::{
    AppState,
    errors::{AppError, AppResult},
    models::{user::AuthClaims, audit::{CreateAuditRequest, GrantAuditRequest}},
    services::audit,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/audit/requests", post(submit_request).get(list_requests))
        .route("/audit/requests/:id/decide", post(decide))
        .route("/audit/logs", get(list_logs))
}

async fn submit_request(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<CreateAuditRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if claims.role != "auditor" {
        return Err(AppError::Forbidden);
    }
    let name = sqlx::query_scalar!("SELECT full_name FROM users WHERE id = ?", claims.user_id)
        .fetch_one(&state.pool).await?;
    let request = audit::submit_request(&state.pool, &claims.user_id, &name, req).await?;
    Ok(Json(serde_json::json!({ "request": request })))
}

async fn decide(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
    Json(body): Json<GrantAuditRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }
    let updated = audit::grant_request(&state.pool, &id, &claims.user_id, &claims.role, body).await?;
    Ok(Json(serde_json::json!({ "request": updated })))
}

async fn list_requests(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
) -> AppResult<Json<serde_json::Value>> {
    if !["admin", "auditor"].contains(&claims.role.as_str()) {
        return Err(AppError::Forbidden);
    }
    let requests = audit::list_requests(&state.pool).await?;
    Ok(Json(serde_json::json!({ "requests": requests })))
}

async fn list_logs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
) -> AppResult<Json<serde_json::Value>> {
    if !["admin", "auditor"].contains(&claims.role.as_str()) {
        return Err(AppError::Forbidden);
    }
    let logs = audit::list_logs(&state.pool).await?;
    Ok(Json(serde_json::json!({ "logs": logs })))
}