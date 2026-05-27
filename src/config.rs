use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url:       String,
    pub jwt_secret:         String,
    pub encryption_key:     String,
    // SoSoValue — two base URLs per their actual API structure
    pub sosovalue_api_url:  String,  // https://api.sosovalue.xyz  (ETF)
    pub sosovalue_news_url: String,  // https://openapi.sosovalue.com (news + currencies)
    pub sosovalue_api_key:  String,  // x-soso-api-key header
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL required")?,
            jwt_secret: std::env::var("JWT_SECRET")
                .context("JWT_SECRET required")?,
            encryption_key: std::env::var("ENCRYPTION_KEY")
                .context("ENCRYPTION_KEY required (32-byte hex)")?,
            sosovalue_api_url: std::env::var("SOSOVALUE_API_URL")
                .unwrap_or_else(|_| "https://api.sosovalue.xyz".into()),
            sosovalue_news_url: std::env::var("SOSOVALUE_NEWS_URL")
                .unwrap_or_else(|_| "https://openapi.sosovalue.com".into()),
            sosovalue_api_key: std::env::var("SOSOVALUE_API_KEY")
                .context("SOSOVALUE_API_KEY required — get it at sosovalue.com/developer")?,
        })
    }
}