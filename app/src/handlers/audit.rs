use axum::{extract::State, Json};
use uuid::Uuid;

use crate::{
    config::AppState,
    domain::audit::generate_audit_report,
    error::{AppError, AppResult},
    middleware::AuthUser,
};

/// POST /api/audit/report
/// Generates a selective transparency audit report.
/// Returns aggregate stats + Merkle root. No individual entries exposed.
pub async fn generate_report(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    // Only admin or auditor role may generate reports
    if auth.role != "admin" && auth.role != "auditor" {
        return Err(AppError::Unauthorized);
    }

    let report = generate_audit_report(&state.db, auth.org_id).await?;

    Ok(Json(serde_json::json!(report)))
}

/// GET /api/audit/reports
/// Lists all previously generated audit report metadata for this org.
pub async fn list_reports(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" && auth.role != "auditor" {
        return Err(AppError::Unauthorized);
    }

    let rows = sqlx::query!(
        r#"
        SELECT id, entry_count, merkle_root, generated_at
        FROM audit_reports
        WHERE org_id = ?
        ORDER BY generated_at DESC
        LIMIT 20
        "#,
        auth.org_id.to_string(),
    )
    .fetch_all(&state.db)
    .await?;

    let reports: Vec<_> = rows
        .into_iter()
        .map(|r| serde_json::json!({
            "id":           r.id,
            "entry_count":  r.entry_count,
            "merkle_root":  r.merkle_root,
            "generated_at": r.generated_at.map(|d| d.to_string()),
        }))
        .collect();

    Ok(Json(serde_json::json!({ "reports": reports })))
}