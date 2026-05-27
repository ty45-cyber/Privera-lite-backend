use axum::{
    extract::{Extension, Path, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use crate::{
    AppState,
    errors::{AppError, AppResult},
    models::{user::AuthClaims, governance::{CreateProposal, CastVote}},
    services::governance,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/governance/proposals", post(create_proposal).get(list_proposals))
        .route("/governance/proposals/:id/vote", post(vote))
        .route("/governance/proposals/:id/results", get(results))
}

async fn create_proposal(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<CreateProposal>,
) -> AppResult<Json<serde_json::Value>> {
    if !["admin", "finance"].contains(&claims.role.as_str()) {
        return Err(AppError::Forbidden);
    }
    let proposal = governance::create_proposal(&state.pool, req, &claims.user_id).await?;
    Ok(Json(serde_json::json!({ "proposal": proposal })))
}

async fn list_proposals(
    State(state): State<Arc<AppState>>,
    Extension(_): Extension<AuthClaims>,
) -> AppResult<Json<serde_json::Value>> {
    let proposals = governance::list_proposals(&state.pool).await?;
    Ok(Json(serde_json::json!({ "proposals": proposals })))
}

async fn vote(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
    Json(ballot): Json<CastVote>,
) -> AppResult<Json<serde_json::Value>> {
    governance::cast_vote(&state.pool, &state.cfg.encryption_key, &id, &claims.user_id, ballot).await?;
    Ok(Json(serde_json::json!({ "message": "Vote cast. Your ballot is encrypted." })))
}

async fn results(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    // Only admin can decrypt tally
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }
    let tally = governance::tally(&state.pool, &state.cfg.encryption_key, &id).await?;
    Ok(Json(serde_json::json!({ "results": tally })))
}