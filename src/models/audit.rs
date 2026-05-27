use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditRequest {
    pub id: String,
    pub auditor_id: String,
    pub auditor_name: String,
    pub resource_type: String, // payroll | treasury | governance 
    pub resource_id: String,
    pub reason: String,
    pub status: String, // pending | granted | denied
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: String,
    pub actor_id: String,
    pub actor_role: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub metadata: Option<String>, // JSON blob
    pub occured_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuditRequest {
    pub resource_type: String,
    pub resource_id: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct GrantAuditRequest {
    pub decision: String, // granted | denied
}
