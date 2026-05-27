use axum::{extract::State, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::AppState,
    domain::auth::issue_token,
    error::{AppError, AppResult},
};

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub org_id: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    if body.password.len() <12 {
        return Err(AppError::Validation("Password must be at least 12 characters.into()"));
    }

    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let password_hash = hash(&body.password, DEFAULT_COST)
    .map_err(|e| AppError::Internal(e.into()))?;

    sqlx::query!(
        "INSERT INTO organizations (id, name) VALUES (?, ?)",
        org_id.to_string(),
        body.org_name,
    )
    .execute(&state.db)
    .await?;

    sqlx::query!(
        "INSERT INTO users (id, org_id, email, password_hash, role) VALUES (?, ?, ?,?, 'admin)",
     user_id.to_string(),
     org_id.to_string(),
     body.email,
     password_hash,   
    )

    .execute(&state.db)
    .await?;

    let token = issue_token(user_id, org_id, "admin")?;

    Ok(Json(AuthResponse {
        token,
        user_id: user_id.to_string(),
        org_id: org_id.to_string(),
    }))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
 State(state): State<AppState>,
 Json(body): Json<LoginRequest>,   
) -> AppResult<Json<AuthResponse>> {
    let user = sqlx::query!(
        "SELECT id, org_id, password_hash, role FROM users WHERE email = ?",
        body.email,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let valid = verify(&body.password, &user.password_hash)
    .map_err(|e| AppError::Internal(e.into()))?;

    if !valid {
        return Err(AppError::Unauthorized);
    }

    let user_id = Uuid::parse_str(&user.id).map_err(|e| AppError::Internal(e.into()))?;
    let org_id = Uuid::parse_str(&user.org_id).map_err(|e| AppError::Internal(e.into()))?;
    let token = issue_token(user_id, org_id, &user.role)?;

    Ok(Json(AuthResponse{
        token,
        user_id: user.id,
        org_id: user.org_id,
    }))

}