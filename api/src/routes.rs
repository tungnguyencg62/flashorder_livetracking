use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;

use crate::{
    handlers::{self, echo, health, hello_world, info, ready},
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/", get(hello_world))
        .route("/info", get(info))
        .route("/echo/{text}", get(echo))
        .route("/ready", get(ready))
        .route("/v1/me", get(handlers::me))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};

    use tower::ServiceExt;

    #[tokio::test]
    async fn request_qua_full_router_co_tracelayer_van_tra_200() {
        let state = AppState::for_test(4000);
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
        let state = AppState::for_test(4000);
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
    #[tokio::test]
    async fn me_qua_router_voi_token_hop_le_tra_ve_200() {
        use axum::http::header::AUTHORIZATION;
        use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
        use serde_json::json;

        let state = AppState::for_test(4000);
        let app = build_router(state);

        let exp = (chrono::Utc::now().timestamp() + 3600) as usize;
        let claims = json!({
            "sub": "11111111-1111-1111-1111-111111111111",
            "session_id": "22222222-2222-2222-2222-222222222222",
            "email": "user@example.com",
            "role": "",
            "iss": "http://localhost:9999/auth/v1",
            "aud": "authenticated",
            "exp": exp,
            "app_metadata": {}
        });
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret("test-secret".as_bytes()),
        )
        .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/me")
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
