use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::Utc;
use crate::{
    errors::{AppError, AppResult},
    models::payroll::*,
    services::encryption,
};

/// Persist a payroll batch and encrypt all salary fields. O(n)
pub async fn create_batch(
    pool: &MySqlPool,
    enc_key: &str,
    name: &str,
    period_start: &str,
    period_end: &str,
    rows: Vec<CsvPayrollRow>,
    created_by: &str,
) -> AppResult<PayrollBatch> {
    let batch_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let count = rows.len() as i32;

    sqlx::query!(
        r#"INSERT INTO payroll_batches (id, name, period_start, period_end, total_records, status, created_by, created_at)
           VALUES (?, ?, ?, ?, ?, 'encrypted', ?, ?)"#,
        batch_id, name, period_start, period_end, count, created_by, now
    )
    .execute(pool)
    .await?;

    // Single-pass encrypt + insert — O(n)
    for row in &rows {
        let rec_id = Uuid::new_v4().to_string();
        let enc_salary = encryption::encrypt(enc_key, &row.gross_salary.to_string())?;
        let enc_ded    = encryption::encrypt(enc_key, &row.deductions.to_string())?;
        let net        = row.gross_salary - row.deductions;
        let enc_net    = encryption::encrypt(enc_key, &net.to_string())?;

        sqlx::query!(
            r#"INSERT INTO payroll_records
               (id, batch_id, employee_id, employee_name, department, encrypted_salary, encrypted_deductions, encrypted_net, currency)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            rec_id, batch_id, row.employee_id, row.employee_name, row.department,
            enc_salary, enc_ded, enc_net, row.currency
        )
        .execute(pool)
        .await?;
    }

    Ok(PayrollBatch {
        id: batch_id,
        name: name.into(),
        period_start: period_start.into(),
        period_end: period_end.into(),
        total_records: count,
        status: "encrypted".into(),
        created_by: created_by.into(),
        created_at: now,
    })
}

/// List all batches. O(n)
pub async fn list_batches(pool: &MySqlPool) -> AppResult<Vec<PayrollBatch>> {
    let batches = sqlx::query_as!(PayrollBatch,
        "SELECT id, name, period_start, period_end, total_records, status, created_by, created_at FROM payroll_batches ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;
    Ok(batches)
}

/// Returns decrypted records for authorized roles (hr, finance, admin). O(n)
pub async fn get_batch_decrypted(
    pool: &MySqlPool,
    enc_key: &str,
    batch_id: &str,
) -> AppResult<Vec<PayrollRecordDecrypted>> {
    let records = sqlx::query_as!(PayrollRecord,
        "SELECT id, batch_id, employee_id, employee_name, department, encrypted_salary, encrypted_deductions, encrypted_net, currency FROM payroll_records WHERE batch_id = ?",
        batch_id
    )
    .fetch_all(pool)
    .await?;

    records.into_iter().map(|r| {
        let gross = encryption::decrypt(enc_key, &r.encrypted_salary)?.parse::<f64>()
            .map_err(|_| AppError::Encryption("Salary parse failed".into()))?;
        let ded = encryption::decrypt(enc_key, &r.encrypted_deductions)?.parse::<f64>()
            .map_err(|_| AppError::Encryption("Deductions parse failed".into()))?;
        let net = encryption::decrypt(enc_key, &r.encrypted_net)?.parse::<f64>()
            .map_err(|_| AppError::Encryption("Net parse failed".into()))?;
        Ok(PayrollRecordDecrypted {
            id: r.id,
            batch_id: r.batch_id,
            employee_id: r.employee_id,
            employee_name: r.employee_name,
            department: r.department,
            gross_salary: gross,
            deductions: ded,
            net_salary: net,
            currency: r.currency,
        })
    }).collect()
}

/// Returns masked records (redacted salaries). O(n)
pub async fn get_batch_masked(pool: &MySqlPool, batch_id: &str) -> AppResult<Vec<PayrollRecordMasked>> {
    let records = sqlx::query_as!(PayrollRecord,
        "SELECT id, batch_id, employee_id, employee_name, department, encrypted_salary, encrypted_deductions, encrypted_net, currency FROM payroll_records WHERE batch_id = ?",
        batch_id
    )
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| PayrollRecordMasked {
        id: r.id,
        batch_id: r.batch_id,
        employee_id: r.employee_id,
        employee_name: r.employee_name,
        department: r.department,
        gross_salary: "••••••".into(),
        deductions: "••••••".into(),
        net_salary: "••••••".into(),
        currency: r.currency,
    }).collect())
}

/// Generates a plaintext audit CSV without storing it (streaming-safe). O(n)
pub async fn generate_audit_export(
    pool: &MySqlPool,
    enc_key: &str,
    batch_id: &str,
) -> AppResult<String> {
    let records = get_batch_decrypted(pool, enc_key, batch_id).await?;
    let mut csv = String::from("employee_id,employee_name,department,gross_salary,deductions,net_salary,currency\n");
    for r in records {
        csv.push_str(&format!(
            "{},{},{},{:.2},{:.2},{:.2},{}\n",
            r.employee_id, r.employee_name, r.department,
            r.gross_salary, r.deductions, r.net_salary, r.currency
        ));
    }
    Ok(csv)
}