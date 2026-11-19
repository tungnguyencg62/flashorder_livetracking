use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;

use crate::{
    handlers::{echo, health, hello_world, info},
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/", get(hello_world))
        .route("/info", get(info))
        .route("/echo/{text}", get(echo))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

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
