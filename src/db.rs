use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use anyhow::Result;

pub async fn init_pool(url: &str) -> Result<MySqlPool> {
    let pool = MySqlPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(std::time::Duration::from_secs(5))
    .connect(url)
    .await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &MySqlPool) -> Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}