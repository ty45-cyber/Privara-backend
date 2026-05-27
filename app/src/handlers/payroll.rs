use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate:;{
    config::AppState,
    domain::payroll::{
        create_payroll_entry, generate_audit_token, list_payroll_for_org, CreatePayrollParams,

    },
    error::{AppError, AppResult},
    middleware::AuthUser,
};

#[derive(Deserialize)]
pub struct CreatePayrollRequest {
    pub contractor_name: String,
    pub amount: f64,
    pub currency: String,
    pub wallet_address: String,
}

#[derive(Serialize)]
pub struct CreatePayrollResponse {
    pub id: String,
    pub zk_commitment: String,
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreatePayrollRequest>,

) -> AppResult<Json<CreatePayrollResponse>> {
    let id = create_payroll_entry(
        &state.db,
        CreatePayrollParams {
            org_id: auth.org_id,
            created_by: auth.user_id,
            contracter_name: body.contractor_name,
            amount: body.amount,
            currency: body.currency,
            wallet_address: body.wallet_address,
        }
    )
    .await?;

    // Fetch commitment for response 
    let row = sqlx::query!(
 "SELECT zk_commitment FROM payroll_entries WHERE id = ?",
        id.to_string()       
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(CreatePayrollResponse {
        id: id.to_string(),
        zk_commitment: row.zk_commitment,
    }))

}
#[derive(Serialize)]
pub struct PayrollListItem {
    pub id: String,
    pub contractor_name: String,
    pub amount: f64,
    pub currency: String,
    pub wallet_address: String,
    pub status: String,
    pub zk_commitment: String,
    pub has_audit_token: bool,
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<PayrollListItem>>> {
    let entries = list_payroll_for_org(&state.db, auth.org_id).await?;

    let items = entries 
    .into_iter()
    .map(|e| PayrollListItem {
        id: e.id.to_string(),
        contractor_name: e.contractor_name,
        amount: e.amount,
        currency: e.currency,
        wallet_address: e.wallet_address,
        status: e.status,
        zk_commitment: e.zk_commitment,
        has_audit_token: e.has_audit_token,

    })
    .collect();
    Ok(Json(items))
}

pub async fn audit(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" && auth.role != "auditor" {
        return Err(AppError::Unauthorized);
    }

    let entry_id = Uuid::parse_str(&id).map_err(|_| AppError::Validation("Invalid ID".into()))?;

    Ok(Json(serde_json::json!({ "audit_token": token})))

}


