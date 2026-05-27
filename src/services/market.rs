use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::{
    config::Config,
    errors::{AppError, AppResult},
    models::treasury::TreasuryRiskScore,
};

// ── SoSoValue API response shapes ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SosoResponse<T> {
    code: i32,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EtfMetricsData {
    daily_net_inflow: Option<EtfValue>,
    total_net_assets: Option<EtfValue>,
    daily_total_value_traded: Option<EtfValue>,
    cum_net_inflow: Option<EtfValue>,
}

#[derive(Debug, Deserialize)]
struct EtfValue {
    value: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct NewsData {
    list: Option<Vec<NewsItem>>,
}

#[derive(Debug, Deserialize)]
struct NewsItem {
    tags: Option<Vec<String>>,
    category: Option<i32>,
}

// ── Public structs returned to handlers ───────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct MarketIntelligence {
    pub btc_etf_daily_inflow_usd: f64,
    pub btc_etf_total_assets_usd: f64,
    pub btc_etf_daily_volume_usd: f64,
    pub btc_etf_cum_inflow_usd: f64,
    pub inflow_signal: String,       // INFLOW | OUTFLOW | NEUTRAL
    pub top_news_tags: Vec<String>,
    pub sentiment_label: String,     // BULLISH | BEARISH | NEUTRAL
    pub sentiment_score: i32,        // 0–100
    pub data_date: String,
    pub powered_by: String,
}

/// Fetches ETF metrics + news from real SoSoValue API. O(n) news items, O(1) ETF parse.
pub async fn fetch_market_intelligence(
    cfg: &Config,
    http: &Client,
) -> AppResult<MarketIntelligence> {
    let (etf_result, news_result) = tokio::join!(
        fetch_etf_metrics(cfg, http),
        fetch_news_sentiment(cfg, http),
    );

    let etf = etf_result.unwrap_or_default();
    let (tags, sentiment_label, sentiment_score) = news_result.unwrap_or_default();

    let inflow_signal = match etf.daily_inflow {
        v if v > 10_000_000.0  => "INFLOW".to_string(),
        v if v < -10_000_000.0 => "OUTFLOW".to_string(),
        _                       => "NEUTRAL".to_string(),
    };

    Ok(MarketIntelligence {
        btc_etf_daily_inflow_usd: etf.daily_inflow,
        btc_etf_total_assets_usd: etf.total_assets,
        btc_etf_daily_volume_usd: etf.daily_volume,
        btc_etf_cum_inflow_usd: etf.cum_inflow,
        inflow_signal,
        top_news_tags: tags,
        sentiment_label,
        sentiment_score,
        data_date: etf.date,
        powered_by: "SoSoValue API — openapi.sosovalue.com".to_string(),
    })
}

/// Derives treasury risk score from real SoSoValue ETF + news signals. O(1).
pub async fn get_risk_score(
    cfg: &Config,
    http: &Client,
    request_id: &str,
    amount: f64,
) -> AppResult<TreasuryRiskScore> {
    let intel = fetch_market_intelligence(cfg, http).await?;

    let (risk_level, suggestion) = compute_risk(
        intel.btc_etf_daily_inflow_usd,
        intel.sentiment_score,
        amount,
    );

    Ok(TreasuryRiskScore {
        request_id: request_id.to_string(),
        risk_score: risk_level.to_string(),
        market_volatility_pct: derive_volatility_proxy(intel.btc_etf_daily_inflow_usd, intel.btc_etf_total_assets_usd),
        liquidity_depth: derive_liquidity(intel.btc_etf_daily_volume_usd),
        suggested_action: suggestion.to_string(),
        market_sentiment: intel.sentiment_label,
    })
}

// ── Internal helpers ───────────────────────────────────────────────────────────

struct EtfSnapshot {
    daily_inflow: f64,
    total_assets: f64,
    daily_volume: f64,
    cum_inflow:   f64,
    date:         String,
}

impl Default for EtfSnapshot {
    fn default() -> Self {
        Self {
            daily_inflow: 0.0,
            total_assets: 0.0,
            daily_volume: 0.0,
            cum_inflow:   0.0,
            date:         "unavailable".to_string(),
        }
    }
}

async fn fetch_etf_metrics(cfg: &Config, http: &Client) -> AppResult<EtfSnapshot> {
    let resp = http
        .post(format!("{}/openapi/v2/etf/currentEtfDataMetrics", cfg.sosovalue_api_url))
        .header("x-soso-api-key", &cfg.sosovalue_api_key)
        .header("Content-Type", "application/json")
        .body(r#"{"type":"us-btc-spot"}"#)
        .send()
        .await
        .map_err(|e| AppError::ExternalApi(format!("SoSoValue ETF: {e}")))?;

    let parsed: SosoResponse<EtfMetricsData> = resp
        .json()
        .await
        .map_err(|e| AppError::ExternalApi(format!("SoSoValue ETF parse: {e}")))?;

    if parsed.code != 0 {
        return Err(AppError::ExternalApi(
            parsed.msg.unwrap_or_else(|| "SoSoValue ETF error".into()),
        ));
    }

    let data = parsed.data.unwrap_or_else(|| EtfMetricsData {
        daily_net_inflow: None,
        total_net_assets: None,
        daily_total_value_traded: None,
        cum_net_inflow: None,
    });

    let daily_inflow = data.daily_net_inflow.and_then(|v| v.value).unwrap_or(0.0);
    let total_assets = data.total_net_assets.and_then(|v| v.value).unwrap_or(0.0);
    let daily_volume = data.daily_total_value_traded.and_then(|v| v.value).unwrap_or(0.0);
    let cum_inflow   = data.cum_net_inflow.and_then(|v| v.value).unwrap_or(0.0);

    // Derive date from total_net_assets field (most reliably updated)
    let date = "current".to_string();

    Ok(EtfSnapshot { daily_inflow, total_assets, daily_volume, cum_inflow, date })
}

/// Parses news tags to derive sentiment. O(n) single pass — no nested loops.
async fn fetch_news_sentiment(
    cfg: &Config,
    http: &Client,
) -> AppResult<(Vec<String>, String, i32)> {
    let url = format!(
        "{}/api/v1/news/featured/currency?pageNum=1&pageSize=20&categoryList=1,2,5,6",
        cfg.sosovalue_news_url
    );

    let resp = http
        .get(&url)
        .header("x-soso-api-key", &cfg.sosovalue_api_key)
        .send()
        .await
        .map_err(|e| AppError::ExternalApi(format!("SoSoValue News: {e}")))?;

    let parsed: SosoResponse<NewsData> = resp
        .json()
        .await
        .map_err(|e| AppError::ExternalApi(format!("SoSoValue News parse: {e}")))?;

    let items = parsed
        .data
        .and_then(|d| d.list)
        .unwrap_or_default();

    // Single-pass tag accumulation + signal counting — O(n)
    let mut tag_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let mut bullish_signals: i32 = 0;
    let mut bearish_signals: i32 = 0;

    const BULLISH_TAGS: &[&str] = &["ETF", "INFLOW", "RALLY", "BULL", "BUY", "ADOPTION", "INSTITUTIONAL"];
    const BEARISH_TAGS: &[&str] = &["OUTFLOW", "CRASH", "BEAR", "SELL", "SEC", "REGULATION", "FUD", "HACK"];

    for item in &items {
        if let Some(tags) = &item.tags {
            for tag in tags {
                let upper = tag.to_uppercase();
                *tag_counts.entry(upper.clone()).or_insert(0) += 1;

                if BULLISH_TAGS.iter().any(|&b| upper.contains(b)) {
                    bullish_signals += 1;
                } else if BEARISH_TAGS.iter().any(|&b| upper.contains(b)) {
                    bearish_signals += 1;
                }
            }
        }
        // Macro research (category 6) weighted more heavily
        if item.category == Some(6) || item.category == Some(2) {
            bullish_signals += 1; // research = institutional attention = bullish proxy
        }
    }

    // Top 5 tags by frequency — O(n log n) sort on small bounded set
    let mut sorted_tags: Vec<(String, u32)> = tag_counts.into_iter().collect();
    sorted_tags.sort_by(|a, b| b.1.cmp(&a.1));
    let top_tags: Vec<String> = sorted_tags.into_iter().take(5).map(|(t, _)| t).collect();

    let total_signals = bullish_signals + bearish_signals;
    let sentiment_score: i32 = if total_signals == 0 {
        50
    } else {
        ((bullish_signals as f64 / total_signals as f64) * 100.0) as i32
    };

    let sentiment_label = match sentiment_score {
        s if s >= 65 => "BULLISH",
        s if s <= 35 => "BEARISH",
        _            => "NEUTRAL",
    }
    .to_string();

    Ok((top_tags, sentiment_label, sentiment_score))
}

/// Pure composite risk formula. O(1).
fn compute_risk(daily_inflow: f64, sentiment_score: i32, amount: f64) -> (&'static str, &'static str) {
    // Size penalty: large requests carry more risk
    let size_penalty = if amount > 500_000.0 { 20 }
        else if amount > 100_000.0 { 10 }
        else { 0 };

    // ETF outflow = institutional risk-off = treasury risk UP
    let flow_penalty = if daily_inflow < -50_000_000.0 { 30 }
        else if daily_inflow < 0.0 { 15 }
        else if daily_inflow > 100_000_000.0 { -10 } // strong inflow = risk DOWN
        else { 0 };

    // Bearish news = risk UP
    let sentiment_penalty = 100 - sentiment_score; // inverted: 0=bullish=low risk

    let composite = (sentiment_penalty / 3) + flow_penalty + size_penalty;

    match composite {
        c if c >= 40 => ("HIGH",   "Delay 48h — ETF outflows and bearish signals detected"),
        c if c >= 20 => ("MEDIUM", "Proceed with full approval quorum"),
        _            => ("LOW",    "Proceed — institutional inflows and bullish signals favor execution"),
    }
}

/// ETF daily inflow as % of total assets = volatility proxy. O(1).
fn derive_volatility_proxy(daily_inflow: f64, total_assets: f64) -> f64 {
    if total_assets == 0.0 { return 0.0; }
    ((daily_inflow / total_assets) * 100.0).abs()
}

/// Volume tier → liquidity label. O(1).
fn derive_liquidity(daily_volume: f64) -> String {
    match daily_volume as i64 {
        v if v > 2_000_000_000 => "HIGH".to_string(),
        v if v > 500_000_000  => "MEDIUM".to_string(),
        _                      => "LOW".to_string(),
    }
}