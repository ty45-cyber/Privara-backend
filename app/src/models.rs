use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid,

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
 pub id: Uuid,
 pub org_id: Uuid,
 pub email: String,
 pub role: String, // admin | member | auditor 
 pub created_at: DateTime<Utc>,   
}

/// Encrypted contractor payment record 
/// Sensitive fields (amount, recipient_wallet) are stored AES-256-GCM encrypted
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PayrollEntry {
    pub id: Uuid,
    pub org_id: Uuid,
    pub created_by: Uuid,
    pub contractor_name_enc: String, // encrypted 
    pub amount_enc: String,
    pub currency: String, 
    pub wallet_address_enc: String, // encrypted 
    pub zk_commitment: String, // public commitment, no value exposed
    pub status: String, // pending | processed | audited 
    pub audit_token: Option<String>, // selective disclosure token
    pub created_at: DateTime<Utc> 
}

/// Private governance proposal.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Proposal {
    pub id: Uuid,
    pub org_id: Uuid,
    pub title_enc: String, // encrypted
    pub description_enc: String, // encrypted
    pub creator_id: Uuid,
    pub quorun: i32,
    pub status: String, // active | passed | rejected | expired
    pub ends_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
 
}

/// Coercion-resistant encrypted vote.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Vote {
    pub id: Uuid,
    pub proposal_id: Uuid,
    pub voter_commitment: String, // ZK commitment of voter identity
    pub vote_enc: String, // encrypted: yes | no | abstain
    pub created_at: DateTime<Utc>,
}