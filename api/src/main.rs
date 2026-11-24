use std::sync::Arc;
mod auth;
mod cache;
mod config;
mod db;
mod error;
mod handlers;
mod repository;
mod revocation;
mod routes;
mod shutdown;
mod state;
use config::Config;

use state::AppState;

use tracing_subscriber::EnvFilter;

use crate::{
    routes::build_router,
    shutdown::{run, shutdown_signal},
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env().unwrap_or_else(|err| {
        eprintln!("Lỗi cấu hình: {err}");
        std::process::exit(1);
    });

    let db = match db::connect(&config.database_url).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("lỗi kết nối database: {e}");
            std::process::exit(1);
        }
    };

    let redis_conn = cache::connect(&config.redis_url).await.unwrap_or_else(|e| {
        eprintln!("Không kết nối được Redis: {e}");
        std::process::exit(1);
    });
    tracing::info!("Đã kết nối Redis");

    let revocation: Arc<dyn revocation::RevocationStore> =
        Arc::new(revocation::RedisRevocationStore::new(redis_conn));

    let state = AppState {
        config: Arc::new(config),
        db: db,
        revocation: revocation,
    };

    let addr = format!("0.0.0.0:{}", state.config.port);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Server đang chạy tại http://{addr}");

    run(listener, app, shutdown_signal()).await;

    tracing::info!("Server đã tắt hoàn toàn.");
}
