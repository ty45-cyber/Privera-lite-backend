use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PayrollBatch {
    pub id: String,
    pub name: String,
    pub period_start: String,
    pub period_end: String,
    pub total_records: i32,
    pub status: String, // draft | encrypted | released | audited
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PayrollRecord {
    pub id: String,
    pub batch_id: String,
    pub employee_id: String,
    pub employee_name: String,
    pub department: String,
    pub encrypted_salary: String,
    pub encrypted_deductions: String,
    pub encrypted_net: String,
    pub currency: String,
}

#[derive(Debug, Serialize)]
pub struct PayrollRecordDecrypted {
    pub id: String,
    pub batch_id: String,
    pub employee_id: String,
    pub employee_name: String,
    pub department: String,
    pub gross_salary: f64,
    pub deductions: f64,
    pub net_salary: f64,
    pub currency: String,
}

#[derive(Debug, Serialize)]
pub struct PayrollRecordMasked {
    pub id: String,
    pub batch_id: String,
    pub employee_id: String,
    pub employee_name: String,
    pub department: String,
    pub gross_salary: String,
    pub deductions: String,
    pub net_salary: String,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
pub struct CsvPayrollRow {
    pub employee_id: String,
    pub employee_name: String,
    pub department: String,
    pub gross_salary: f64,
    pub deductions: f64,
    pub currency: String,
}
