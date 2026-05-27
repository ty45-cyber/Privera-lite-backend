use axum::{extract::State, routing::get, Json, Router};
use std::sync::Arc;
use reqwest::Client;
use crate::{AppState, errors::AppResult, services::market};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/market/intelligence", get(intelligence))
        .route("/market/etf", get(etf_summary))
}

/// Full SoSoValue intelligence — ETF flows + news sentiment. Shown on dashboard.
async fn intelligence(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let http = Client::new();
    let intel = market::fetch_market_intelligence(&state.cfg, &http).await?;
    Ok(Json(serde_json::json!({ "intelligence": intel })))
}

/// Lightweight ETF summary for dashboard widget.
async fn etf_summary(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let http = Client::new();
    let intel = market::fetch_market_intelligence(&state.cfg, &http).await?;
    Ok(Json(serde_json::json!({
        "daily_inflow": intel.btc_etf_daily_inflow_usd,
        "total_assets": intel.btc_etf_total_assets_usd,
        "inflow_signal": intel.inflow_signal,
        "sentiment": intel.sentiment_label,
        "sentiment_score": intel.sentiment_score,
        "top_tags": intel.top_news_tags,
        "powered_by": intel.powered_by
    })))
}