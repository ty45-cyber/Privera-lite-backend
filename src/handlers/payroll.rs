use axum::{
    extract::{Extension, Multipart, Path, State},
    routing::{get, post},
    Json, Router,
    response::IntoResponse,
    http::header,
};
use std::sync::Arc;
use crate::{
    AppState,
    errors::{AppError, AppResult},
    models::user::AuthClaims,
    services::payroll,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/payroll/upload", post(upload))
        .route("/payroll/batches", get(list_batches))
        .route("/payroll/batches/:id", get(get_batch))
        .route("/payroll/batches/:id/audit-export", get(audit_export))
}

async fn upload(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    if !["admin", "hr"].contains(&claims.role.as_str()) {
        return Err(AppError::Forbidden);
    }

    let mut name = String::new();
    let mut period_start = String::new();
    let mut period_end = String::new();
    let mut csv_bytes: Vec<u8> = vec![];

    while let Some(field) = multipart.next_field().await.map_err(|_| AppError::Validation("Multipart error".into()))? {
        match field.name().unwrap_or("") {
            "name"         => name = field.text().await.unwrap_or_default(),
            "period_start" => period_start = field.text().await.unwrap_or_default(),
            "period_end"   => period_end = field.text().await.unwrap_or_default(),
            "file"         => csv_bytes = field.bytes().await.unwrap_or_default().to_vec(),
            _ => {}
        }
    }

    if csv_bytes.is_empty() {
        return Err(AppError::Validation("No CSV file provided".into()));
    }

    let mut reader = csv::Reader::from_reader(csv_bytes.as_slice());
    let rows: Vec<crate::models::payroll::CsvPayrollRow> = reader
        .deserialize()
        .collect::<Result<_, _>>()
        .map_err(|e| AppError::Validation(format!("CSV parse: {e}")))?;

    let batch = payroll::create_batch(
        &state.pool, &state.cfg.encryption_key,
        &name, &period_start, &period_end, rows, &claims.user_id,
    ).await?;

    Ok(Json(serde_json::json!({ "batch": batch, "message": "Payroll encrypted and stored" })))
}

async fn list_batches(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<AuthClaims>,
) -> AppResult<Json<serde_json::Value>> {
    let batches = payroll::list_batches(&state.pool).await?;
    Ok(Json(serde_json::json!({ "batches": batches })))
}

async fn get_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    match claims.role.as_str() {
        "admin" | "hr" | "finance" => {
            let records = payroll::get_batch_decrypted(&state.pool, &state.cfg.encryption_key, &id).await?;
            Ok(Json(serde_json::json!({ "records": records, "view": "decrypted" })))
        }
        _ => {
            let records = payroll::get_batch_masked(&state.pool, &id).await?;
            Ok(Json(serde_json::json!({ "records": records, "view": "masked" })))
        }
    }
}

async fn audit_export(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    if !["admin", "auditor"].contains(&claims.role.as_str()) {
        return Err(AppError::Forbidden);
    }
    let csv = payroll::generate_audit_export(&state.pool, &state.cfg.encryption_key, &id).await?;
    Ok((
        [(header::CONTENT_TYPE, "text/csv"), (header::CONTENT_DISPOSITION, "attachment; filename=\"audit.csv\"")],
        csv,
    ))
}