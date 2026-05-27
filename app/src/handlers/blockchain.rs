use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    config::AppState,
    error::{AppError, AppResult},
    middleware::AuthUser,
};

/// GET /api/blockchain/network
/// Returns live Fhenix Nitrogen network status — used by the frontend status bar.
pub async fn network_status(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let info = state
        .fhenix
        .network_info()
        .await
        .map_err(|e| AppError::Internal(e))?;

    Ok(Json(serde_json::json!({
        "chain_id": info.chain_id,
        "rpc_url": info.rpc_url,
        "explorer_url": info.explorer_url,
        "latest_block": info.latest_block,
        "network": "Fhenix Nitrogen Testnet",
        "status": "online"
    })))
}

#[derive(Deserialize)]
pub struct VerifyTxRequest {
    pub tx_hash: String,
}

#[derive(Serialize)]
pub struct VerifyTxResponse {
    pub tx_hash: String,
    pub success: bool,
    pub explorer_url: String,
}

/// POST /api/blockchain/verify-tx
/// Frontend submits a tx hash after an on-chain payroll entry or vote.
/// Backend verifies it succeeded on Fhenix Nitrogen and records it in MySQL.
pub async fn verify_tx(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(body): Json<VerifyTxRequest>,
) -> AppResult<Json<VerifyTxResponse>> {
    if body.tx_hash.is_empty() || !body.tx_hash.starts_with("0x") {
        return Err(AppError::Validation("Invalid tx hash format".into()));
    }

    let success = state
        .fhenix
        .verify_tx_success(&body.tx_hash)
        .await
        .map_err(|e| AppError::Internal(e))?;

    let explorer_url = format!(
        "https://explorer.nitrogen.fhenix.zone/tx/{}",
        body.tx_hash
    );

    // Persist the on-chain record to MySQL for audit trail
    sqlx::query!(
        r#"
        INSERT INTO onchain_txs (tx_hash, status, explorer_url, submitted_by)
        VALUES (?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE status = VALUES(status)
        "#,
        body.tx_hash,
        if success { "success" } else { "pending" },
        explorer_url,
        "system",
    )
    .execute(&state.db)
    .await?;

    Ok(Json(VerifyTxResponse {
        tx_hash: body.tx_hash,
        success,
        explorer_url,
    }))
}

#[derive(Deserialize)]
pub struct BalanceRequest {
    pub address: String,
}

/// GET /api/blockchain/balance?address=0x...
/// Returns tFHE balance for a wallet — shown in the dashboard header.
pub async fn wallet_balance(
    State(state): State<AppState>,
    _auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<BalanceRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let balance = state
        .fhenix
        .get_balance(&params.address)
        .await
        .map_err(|e| AppError::Internal(e))?;

    Ok(Json(serde_json::json!({
        "address": params.address,
        "balance": balance,
        "network": "Fhenix Nitrogen"
    })))
}