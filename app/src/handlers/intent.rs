use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::AppState,
    domain::intent::{fetch_pending_intents, expire_stale_intents},
    error::{AppError, AppResult},
    middleware::AuthUser,
};

#[derive(Deserialize)]
pub struct CreateIntentRequest {
    pub payee_address:    String,
    pub currency:         String,
    pub deadline_seconds: u64,
}

/// POST /api/intents
/// Create a new encrypted disbursement intent.
/// The encrypted amount lives on-chain — this endpoint stores metadata only.
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateIntentRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Unauthorized);
    }
    if body.payee_address.len() != 42 || !body.payee_address.starts_with("0x") {
        return Err(AppError::Validation("Invalid payee address".into()));
    }
    if body.deadline_seconds == 0 || body.deadline_seconds > 604_800 {
        return Err(AppError::Validation("Deadline must be 1s–7days".into()));
    }

    let id = Uuid::new_v4().to_string();
    let deadline = chrono::Utc::now()
        + chrono::Duration::seconds(body.deadline_seconds as i64);

    sqlx::query!(
        r#"
        INSERT INTO payroll_intents
            (id, org_id, payee_address, currency, process_deadline)
        VALUES (?, ?, ?, ?, ?)
        "#,
        id,
        auth.org_id.to_string(),
        body.payee_address,
        body.currency,
        deadline,
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "intent_id": id,
        "status": "pending",
        "deadline": deadline.to_rfc3339(),
    })))
}

/// GET /api/intents
/// List pending intents for this org.
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" && auth.role != "auditor" {
        return Err(AppError::Unauthorized);
    }

    let intents = fetch_pending_intents(&state.db, auth.org_id).await?;

    let items: Vec<_> = intents
        .into_iter()
        .map(|i| serde_json::json!({
            "id":               i.id,
            "payee_address":    i.payee_address,
            "currency":         i.currency,
            "on_chain_id":      i.on_chain_id,
            "status":           i.status,
            "process_deadline": i.process_deadline.to_rfc3339(),
            "created_at":       i.created_at.to_rfc3339(),
        }))
        .collect();

    Ok(Json(serde_json::json!({ "intents": items, "count": items.len() })))
}

/// POST /api/intents/confirm-onchain
/// Frontend calls this after PrivaraIntentQueue.submitIntent() succeeds on-chain.
/// Links the off-chain intent record to its on-chain ID.
#[derive(Deserialize)]
pub struct ConfirmOnchainRequest {
    pub intent_id:    String,
    pub on_chain_id:  u64,
    pub tx_hash:      String,
}

pub async fn confirm_onchain(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<ConfirmOnchainRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Unauthorized);
    }

    sqlx::query!(
        r#"
        UPDATE payroll_intents
        SET on_chain_id = ?, settlement_tx = ?
        WHERE id = ? AND org_id = ?
        "#,
        body.on_chain_id,
        body.tx_hash,
        body.intent_id,
        auth.org_id.to_string(),
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "status": "confirmed",
        "on_chain_id": body.on_chain_id,
        "tx_hash": body.tx_hash,
    })))
}