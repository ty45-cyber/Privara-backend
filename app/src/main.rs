mod blockchain;
mod config;
mod crypto;
mod db;
mod domain;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;

use axum::Router;
use sqlx::mysql::MySqlPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use blockchain::fhenix::FhenixClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "privara=debug,axum=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MySqlPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let fhenix = FhenixClient::new();

    // Log Fhenix network info on startup
    match fhenix.network_info().await {
        Ok(info) => tracing::info!(
            "Fhenix Nitrogen connected — chain_id={} latest_block={}",
            info.chain_id,
            info.latest_block
        ),
        Err(e) => tracing::warn!("Fhenix RPC unreachable at startup: {}", e),
    }

    let state = config::AppState { db: pool, fhenix };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = routes::build_router(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Privara backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}