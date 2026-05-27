use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use uuid::Uuid;

use crate::{
    domain::auth::verify_token,
    error::{AppError, AppResult},
};

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub role: String,

}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where 
S: Send + Sync,
{

    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> AppResult<Self> {
        let auth_header = parts 
        .headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

        let token = auth_header
        .strip_prefix("Bearer")
        .ok_or(AppError::Unauthorized)?;

        let claims = verify_token(token)?;

        Ok(AuthUser {
            user_id: Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?,
            org_id: Uuid::parse_str(&claims.org).map_err(|_| AppError::Unauthorized)?,
            role: claims.role,

        })
    }
}