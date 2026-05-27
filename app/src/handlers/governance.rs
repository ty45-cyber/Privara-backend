use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    config::AppState,
    domain::governance::{
        cast_vote, create_proposal, list_proposals, CreateProposalParams,
    }
    error::{AppError, AppResult},
    middleware::AuthUser,
};

#[derive(Deserialize)]
pub struct CreateProposalRequest {
    pub title: String,
    pub description: String,
    pub quorum: i32,
    pub duration_hours: i64,
}

#[derive(Serialize)]
pub struct ProposalResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub quorum: i32,
    pub status: String,
    pub ends_at; String,
    pub vote_count: i64,
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateProposalRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let id = create_proposal(
        &state.db,
        CreateProposalParam {
            org_id: auth.org_id,
            creator_id: auth.user_id,
            title: body.title,
            description: body.description,
            quorum: body.quorum,
            duration_hours: body.duration_hours,
        },

    )
    .await?;

    Ok(Json(serde_json::json!({"proposal_id": id.to_string()})))
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<ProposalResponse>>> {
    let proposals = list_proposals(&state.db, auth.org_id).await?;

    let views = proposals
    .into_iter()
    .map(|p| ProposalResponse {
        id: p.id.to_string(),
        title: p.title,
        description: p.description,
        quorum: p.quorum,
        status: p.status,
        ends_at: p.ends_at.to_rfc3339(),
        vote_count: p.vote_count,
    })
    .collect();

    Ok(Json(views))
}

#[derive(Deserialize)]
pub struct CastVoteRequest {
    pub proposal_id: String,
    pub vote: String,
}

pub async fn vote(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CastVoteRequest>,

) -> AppResult<Json<serde_json::Value>> {
    let proposal_id = uuid::Uuid::parse_str(&body.proposal_id)
    .map_err(|_| AppError::Validation("Invalid proposal ID".into()))?;

    cast_vote(&state.db, proposal_id, auth.user_id, &body.vote).await?;

    Ok(Json(serde_json::json!({ "status": "vote_recorded"})))
}