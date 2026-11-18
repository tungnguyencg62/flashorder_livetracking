use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
mod config;
use config::Config;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = Config::from_env().unwrap_or_else(|err| {
        eprintln!("Lỗi cấu hình: {err}");
        std::process::exit(1);
    });

    let state = AppState {
        config: Arc::new(config),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/", get(hello_world))
        .route("/info", get(info))
        .with_state(state.clone());

    let addr = format!("0.0.0.0:{}", state.config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Server đang chạy tại http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
async fn hello_world() -> &'static str {
    "Hello, FlashOrder!"
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[derive(Serialize)]
struct InfoResponse {
    port: u16,
}

async fn info(State(state): State<AppState>) -> Json<InfoResponse> {
    Json(InfoResponse {
        port: state.config.port,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn info_tra_ve_dung_port_tu_state() {
        let state = AppState {
            config: Arc::new(Config { port: 4000 }),
        };

        let Json(response) = info(State(state)).await;

        assert_eq!(response.port, 4000);
    }
}
