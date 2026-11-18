use axum::{Json, Router, routing::get};
use serde::Serialize;
mod config;
use config::Config;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = Config::from_env().unwrap_or_else(|err| {
        eprintln!("Lỗi cấu hình: {err}");
        std::process::exit(1);
    });
    let app = Router::new()
        .route("/health", get(health))
        .route("/", get(hello_world));

    let addr = format!("0.0.0.0:{}", config.port);
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
