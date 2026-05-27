use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::Utc;
use crate::{
    errors::{AppError, AppResult},
    models::governance::*,
    services::encryption,
};

pub async fn create_proposal(
    pool: &MySqlPool,
    req: CreateProposal,
    created_by: &str,
) -> AppResult<Proposal> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    sqlx::query!(
        r#"INSERT INTO proposals (id, title, description, status, created_by, voting_ends_at, created_at)
           VALUES (?, ?, ?, 'active', ?, ?, ?)"#,
        id, req.title, req.description, created_by, req.voting_ends_at, now
    )
    .execute(pool)
    .await?;

    Ok(Proposal {
        id, title: req.title, description: req.description,
        status: "active".into(), created_by: created_by.into(),
        voting_ends_at: req.voting_ends_at, created_at: now,
    })
}

pub async fn cast_vote(
    pool: &MySqlPool,
    enc_key: &str,
    proposal_id: &str,
    voter_id: &str,
    ballot: CastVote,
) -> AppResult<()> {
    if !["yes", "no", "abstain"].contains(&ballot.vote.as_str()) {
        return Err(AppError::Validation("Vote must be yes, no, or abstain".into()));
    }

    let status = sqlx::query_scalar!(
        "SELECT status FROM proposals WHERE id = ?", proposal_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Proposal {proposal_id}")))?;

    if status != "active" {
        return Err(AppError::Validation("Proposal is not accepting votes".into()));
    }

    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM votes WHERE proposal_id = ? AND voter_id = ?",
        proposal_id, voter_id
    )
    .fetch_one(pool)
    .await?;

    if existing > 0 {
        return Err(AppError::Conflict("Already voted on this proposal".into()));
    }

    let vote_id = Uuid::new_v4().to_string();
    let encrypted = encryption::encrypt(enc_key, &ballot.vote)?;
    let now = Utc::now();

    sqlx::query!(
        "INSERT INTO votes (id, proposal_id, voter_id, encrypted_vote, cast_at) VALUES (?, ?, ?, ?, ?)",
        vote_id, proposal_id, voter_id, encrypted, now
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn tally(
    pool: &MySqlPool,
    enc_key: &str,
    proposal_id: &str,
) -> AppResult<VoteResult> {
    let votes = sqlx::query_as!(Vote,
        "SELECT id, proposal_id, voter_id, encrypted_vote, cast_at FROM votes WHERE proposal_id = ?",
        proposal_id
    )
    .fetch_all(pool)
    .await?;

    let (mut yes, mut no, mut abstain) = (0i64, 0i64, 0i64);

    for v in &votes {
        match encryption::decrypt(enc_key, &v.encrypted_vote)?.as_str() {
            "yes"     => yes += 1,
            "no"      => no += 1,
            "abstain" => abstain += 1,
            _         => {}
        }
    }

    let total = yes + no + abstain;
    let outcome = if yes > no { "passed" } else if no > yes { "rejected" } else { "tied" };

    Ok(VoteResult {
        proposal_id: proposal_id.into(),
        total_votes: total,
        yes_count: yes,
        no_count: no,
        abstain_count: abstain,
        outcome: outcome.into(),
    })
}

pub async fn list_proposals(pool: &MySqlPool) -> AppResult<Vec<Proposal>> {
    let rows = sqlx::query_as!(Proposal,
        "SELECT id, title, description, status, created_by, voting_ends_at, created_at FROM proposals ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
