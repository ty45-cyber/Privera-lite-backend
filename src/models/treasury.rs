use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TreasuryRequest {
    pub id: String,
    pub title: String,
    pub amount: f64,
    pub currency: String,
    pub purpose: String,
    pub risk_level: String, // low | medium | high
    pub status: String, // pending | approved | rejected | executed
    pub required_approvals: i32,
    pub current_approvals: i32,
    pub requested_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TreasuryApproval {
    pub id: String,
    pub request_id: String,
    pub approver_name: String,
    pub decision: String, // approved | rejected
    pub note: Option<String>,
    pub decided_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTreasuryRequest {
    pub title: String,
    pub amount: f64,
    pub currency: String,
    pub purpose: String,
    pub required_approvals: i32,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalDecision {
    pub decision: String, // approved | rejected
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TreasuryRiskScore {
    pub request_id: String,
    pub risk_score: String,
    pub market_volatility_pct: f64,
    pub liquidity_depth: String,
    pub suggested_action: String,
    pub market_sentiment: String,
}
