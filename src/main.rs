use axum::{Router, middleware};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod errors;
mod handlers;
mod models;
mod services;
mod middleware as mw;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new(
        std::env::var("RUST_LOG").unwrap_or_else(|_| "privara_lite=debug, tower_http=debug ".into()),
    ))
    .with(tracing_subscriber::fmt::layer())
    .init();

    dotenvy::dotenv().ok();

    let cfg = config::Config::from_env()?;
    let pool = db::init_pool(&cfg.database_url).await?;
    db::run_migrations(&pool).await?;

    let state = std::sync::Arc::new(AppState {pool, cfg});

    let public_routes = Router::new()
    .merge(handlers::auth::router());

    let protected_routes = Router::new()
    .merge(handlers::payroll::router())
    .merge(handlers::treasury::router())
    .merge(handlers::audit::router())
    .merge(handlers::market::router())
    .layer(middleware::from_fn_with_state(
        state.clone(),
        mw::auth::require_auth,
    ));
    let app = Router::new()
    .merge(public_routes)
    .merge(protected_routes)
    .merge(handlers::governance::router())
    .layer(
        CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any),
    )
    .layer(TraceLayer::new_for_http())
    .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Privara Lite listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())


}

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::MySqlPool,
    pub cfg: config::Config
}
