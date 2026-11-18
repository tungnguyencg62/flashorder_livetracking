use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::Serialize;
mod config;
mod error;
use config::Config;
use error::AppError;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/", get(hello_world))
        .route("/info", get(info))
        .route("/echo/{text}", get(echo))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

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

    let state = AppState {
        config: Arc::new(config),
    };

    let addr = format!("0.0.0.0:{}", state.config.port);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Server đang chạy tại http://{addr}");

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

#[derive(Serialize)]
struct EchoResponse {
    message: String,
}
async fn echo(Path(text): Path<String>) -> Result<Json<EchoResponse>, AppError> {
    if text.is_empty() || text.chars().count() > 50 {
        return Err(AppError::Validation("Text không được để trống".to_string()));
    }
    Ok(Json(EchoResponse { message: text }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn info_tra_ve_dung_port_tu_state() {
        let state = AppState {
            config: Arc::new(Config { port: 4000 }),
        };
        let Json(response) = info(State(state)).await;
        assert_eq!(response.port, 4000);
    }

    #[tokio::test]
    async fn request_qua_full_router_co_tracelayer_van_tra_200() {
        let state = AppState {
            config: Arc::new(Config { port: 4000 }),
        };
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn echo_hop_le_tra_ve_ok() {
        let result = echo(Path("xin chào".to_string())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn echo_rong_tra_ve_loi_validation() {
        let result = echo(Path(String::new())).await;

        match result {
            Err(AppError::Validation(_)) => {}
            _ => panic!("phải trả về AppError::Validation khi text rỗng"),
        }
    }

    #[tokio::test]
    async fn echo_qua_router_tra_ve_400_khi_qua_dai() {
        let state = AppState {
            config: Arc::new(Config { port: 4000 }),
        };
        let app = build_router(state);

        let text_qua_dai = "a".repeat(51);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/echo/{text_qua_dai}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
