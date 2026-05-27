use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    config::AppState,
    handlers::{auth, blockchain, governance, payroll},
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Auth
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        // Payroll (off-chain encrypted)
        .route("/api/payroll", post(payroll::create).get(payroll::list))
        .route("/api/payroll/:id/audit", post(payroll::audit))
        // Governance (off-chain encrypted)
        .route("/api/governance/proposals", post(governance::create).get(governance::list))
        .route("/api/governance/vote", post(governance::vote))
        // Blockchain — Fhenix Nitrogen
        .route("/api/blockchain/network", get(blockchain::network_status))
        .route("/api/blockchain/verify-tx", post(blockchain::verify_tx))
        .route("/api/blockchain/balance", get(blockchain::wallet_balance))
        // Health
        .route("/health", get(|| async { "Privara OK" }))
        .with_state(state)
}