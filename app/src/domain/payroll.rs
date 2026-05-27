use sqlx:MySqlPool;
use uuid::Uuid;

use crate::{
    crypto::{decrypt, encrypt, zk_commit},
    error::{AppError, AppResult},
    models::PayrollEntry,
};

pub sruct CreatePayrollParams {
    pub org_id: Uuid,
    pub created_by: Uuid,
    pub contracter_name: String,
    pub amount: f64,
    pub currency: String,
    pub wallet_address: String,
}

pub struct PayrollView {
    pub id: Uuid,
    pub contractor_name: String,
    pub amount: f64,
    pub currency: String,
    pub wallet_address: String,
    pub status: String,
    pub zk_commitment: String,
    pub has_audit_token: bool,
}

pub async fn create_payroll_entry(
db: &MySqlPool,
params: CreatePayrollParams,    
) -> AppResult<Uuid {
    if params.amount <= 0.0 {
        return Err(AppError::Validation("Amount must be positive".into()));
    }
    if params.wallet_address.is_empty() {
        return Err(AppError::Validation("Wallet address is required".into()));
    }

    let id = Uuid::new_v4();
    let salt  = Uuid::new_v4().to_string();
    let commitment = zk_commit(&params.amount.to_string(), &salt);

    let name_enc = encrypt(&params.contractor_name, "payroll:name")?;
    let amount_enc = encrypt(&params.amount.to_string(), "payroll:amount")?;
    let wallet_enc = encrypt(&params.wallet_address, "payroll:wallet")?;

 sqlx::query!(
  r#"
        INSERT INTO payroll_entries
            (id, org_id, created_by, contractor_name_enc, amount_enc, currency,
             wallet_address_enc, zk_commitment, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'pending')
        "#, 
        id.to_string(),
        params.org_id.to_string(),
        params.created_by.to_string(),
        name_enc,
        amount_enc,
        params.currency,
        wallet_enc,
        commitment, 
 )   
 .execute(db)
 .await?;

 Ok(id)

}

pub async fn list_payroll_for_org(
    db: &MySqlPool,
    org_id: Uuid,
) -> AppResult<Vec<PayrollView>> {
    let rows = sqlx::query!(
    r#"
        SELECT id, contractor_name_enc, amount_enc, currency,
               wallet_address_enc, status, zk_commitment, audit_token
        FROM payroll_entries
        WHERE org_id = ?
        ORDER BY created_at DESC
        "#,  
        org_id.to_string(),  
    )
    .fetch_all(db)
    .await?;

    let mut entries = Vec::with_capacity(rows.len());

    for row in row {
        let contractor_name = decrypt(&row.contractor_name_enc, "payroll:name")?;
        let amount_str = decrypt(&row.amount_enc, "payroll:amount")?;
        let wallet_address = decrypt(&row.wallet_address_enc, "payroll:wallet")?;
        let amount = amount_str.parse::<f64>().unwrap_or(0.0);

        entries.push(PayrollView {
            id: Uuid:;parse_str(&row.id).map_err(|e| AppError::Internal(e.into()))?,
            contractor_name,
            amount,
            currency: row.currency,
            wallet_address,
            status: row.status,
            zk_commitmnet: row.zk_commitment,
            has_audit_token: row.audit_token.is_some(),
        });

    }
    Ok(entries)
}

pub async fn generate_audit_token(
db: &MySqlPool,
entry_id: Uuid,
org_id: Uuid,

) -> AppResult<String> {
    let token = Uuid::new_v4().to_string();

    let affected = sqlx::query!(
 r#"
        UPDATE payroll_entries
        SET audit_token = ?, status = 'audited'
        WHERE id = ? AND org_id = ?
        "#,
        token,
        entry_id.to_string(),
        org_id.to_string(),       
    )
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(token)
}