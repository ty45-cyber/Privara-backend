use chrono::{Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;


use crate::{
    crypto::{decrypt, encrypt, zk_commit},
    error::{AppError, AppResult},
};

pub struct CreateProposalParams {
pub id: Uuid,
pub title: String,
pub description: String,
pub quorum: i32,
pub status: String,
pub ends_at: chrono::DateTime<Utc>,
pub vote_count: i64,    
}

pub async fn create_proposal(
    db: &MySqlPool,
    params: CreateProposalParams,
) -> AppResult<Uuid> {
    if params.title.trim().is_empty() {
        return Err(AppError::Validation("Proposal title is required".into()));
    }
    if params.quorum <1 {
        return Err(AppError::Validation("Quorum musat be 1".into()));
    }

    let id = Uuid::new_v4();
    let ends_at = Utc::now() + Duration::hours(params.duration_hours);
    let title_enc = encrypt(&params.title, "governance:title")?;

    sqlx::query!(
  r#"
        INSERT INTO proposals
            (id, org_id, creator_id, title_enc, description_enc, quorum, status, ends_at)
        VALUES (?, ?, ?, ?, ?, ?, 'active', ?)
        "#,
        id.to_string(),
        params.org_id.to_string(),
        params.creator_id.to_string(),
        title_enc,
        desc_enc,
        params.quorum,
        ends_at,      
    )
    .execute(db)
    .await?;

    Ok(id)
}

pub async fn list_proposal(
    db: &MySqlPool,
    org_id: Uuid,
) -> AppResult<Vec<ProposalView>> {
    let rows = sqlx::query!(
 r#"
        SELECT p.id, p.title_enc, p.description_enc, p.quorum, p.status, p.ends_at,
               COUNT(v.id) as vote_count
        FROM proposals p
        LEFT JOIN votes v ON v.proposal_id = p.id
        WHERE p.org_id = ?
        GROUP BY p.id
        ORDER BY p.created_at DESC
        "#,
        org_id.to_string(),       
    )
    .fetch_all(db)
    .await?;

    let mut views = Vec::with_capacity(rows.len());

    for row in rows {
        let title = decrypt(&row.title_enc, "governance::title")?;
        let description = decrypt(&row.description_enc, "governance:description")?;

        views.push(ProposalView {
            id: Uuid::parse_str(&row.id).map_err(|e| AppError::Internal(e.into()))?,
            title,
            description,
            quorum: row.quorum,
            status: row.status,
            ends_at: row.ends_at.and_utc(),
            vote_count: row.vote_count.unwrap_or(0)
        });
    }
    Ok(views)
}

pub async fn cast_vote(
    db: &MySqlPool,
    proposal_id: Uuid,
    voter_id: Uuid,
    vote_value: &str, // yes | no | abstain
) -> AppResult<()> {
    if !["yes", "no", "abstain"].contains(&vote_value) {
        return Err(AppError::Validation("Vote must be: ye, no, or abstain".into()));
    }

    // Check proposal is still active 
    let proposal = sqlx::query!(
        "SELECT status, ends_at FROM proposals WHERE id = ?",
        proposal_id.to_string()
    )
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)?;

    if proposal.status != "active" {
     return Err(AppError::Validation"Proposal is no longer active".into());   
    }
    if proposal.ends_at.and_utc() <Utc::now() {
        return Err(AppError::Validation("Proposal voting period has ended".into()));
    }

    // ZK voter commitment - voter identity is hashed, not stored raw
    let salt = Uuid::new_v4().to_string();
    let voter_commitment = zk_commit(&voter_id.to_string(), &salt);
    let vote_enc = encrypt(vote_value, "governance:vote")?;

    sqlx::query!(
 r#"
        INSERT INTO votes (id, proposal_id, voter_commitment, vote_enc)
        VALUES (?, ?, ?, ?)
        "#,
        Uuid::new_v4().to_string(),
        proposal_id.to_string(),
        voter_commitment,
        vote_enc,       
    )
    .execute(db)
    .await?;

    Ok(())
}