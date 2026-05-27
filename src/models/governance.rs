use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String, // active | closed | passed | rejected
    pub created_by: String,
    pub voting_ends_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vote {
    pub id: String,
    pub proposal_id: String,
    pub voter_id: String,
    pub encrypted_vote: String,
    pub cast_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct VoteResult {
    pub proposal_id: String,
    pub total_votes: i64,
    pub yes_count: i64,
    pub no_count: i64,
    pub abstain_count: i64,
    pub outcome: String, // passed | rejected | tied
}

#[derive(Debug, Deserialize)]
pub struct CreateProposal {
    pub title: String,
    pub description: String,
    pub voting_ends_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CastVote {
    pub vote: String, // yes | no | abstain
}
