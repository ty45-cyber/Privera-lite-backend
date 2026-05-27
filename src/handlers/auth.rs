use axum::{extract::State, routing::post, Json, Router};
use std::sync::Arc;
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;
use chrono::Utc;
use crate::{
    AppState,
    errors::{AppError, AppResult},
    models::user::{RegisterRequest, LoginRequest, AuthResponse},
    services::auth::issue_token,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE email = ?", req.email
    )
    .fetch_one(&state.pool)
    .await?;

    if existing > 0 {
        return Err(AppError::Conflict("Email already registered".into()));
    }

    let hash = hash(&req.password, DEFAULT_COST)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Bcrypt: {e}")))?;

    let id = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO users (id, email, password_hash, full_name, role, is_active, created_at) VALUES (?, ?, ?, ?, ?, true, ?)",
        id, req.email, hash, req.full_name, req.role.to_string(), Utc::now()
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "message": "Registered", "user_id": id })))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let user = sqlx::query!(
        "SELECT id, password_hash, role, full_name, is_active FROM users WHERE email = ?",
        req.email
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if !user.is_active {
        return Err(AppError::Unauthorized);
    }

    let valid = verify(&req.password, &user.password_hash)
        .map_err(|_| AppError::Unauthorized)?;
    if !valid {
        return Err(AppError::Unauthorized);
    }

    let token = issue_token(&user.id, &user.role, &state.cfg.jwt_secret)?;
    Ok(Json(AuthResponse { token, user_id: user.id, role: user.role, full_name: user.full_name }))
}