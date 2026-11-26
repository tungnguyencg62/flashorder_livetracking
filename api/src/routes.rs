use axum::{
    Router,
    routing::{get, post},
};
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
        .route("/v1/merchant-only", get(handlers::merchant_only_demo))
        .route("/v1/auth/logout", post(handlers::logout))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::header::AUTHORIZATION;
    use axum::http::{Request, StatusCode};
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use serde_json::json;
    use tower::ServiceExt;
    use uuid::Uuid;

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

    fn make_token(secret: &str, exp_offset_seconds: i64) -> String {
        let exp = (chrono::Utc::now().timestamp() + exp_offset_seconds) as usize;
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

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    async fn call_me_path(app: Router, path: &str, token: &str) -> StatusCode {
        app.oneshot(
            Request::builder()
                .uri(path)
                .header(AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
    }

    #[sqlx::test]
    async fn me_qua_router_voi_token_hop_le_tra_ve_200(pool: sqlx::PgPool) {
        let app = build_router(AppState::for_test_with_db(4000, pool));
        let token = make_token("test-secret", 3600);

        assert_eq!(call_me_path(app, "/v1/me", &token).await, StatusCode::OK);
    }

    #[tokio::test]
    async fn me_qua_router_voi_sai_chu_ky_tra_ve_401() {
        let app = build_router(AppState::for_test(4000));
        let token = make_token("secret-khac-voi-config", 3600);

        assert_eq!(
            call_me_path(app, "/v1/me", &token).await,
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn me_qua_router_voi_token_het_han_tra_ve_401() {
        let app = build_router(AppState::for_test(4000));
        let token = make_token("test-secret", -3600);

        assert_eq!(
            call_me_path(app, "/v1/me", &token).await,
            StatusCode::UNAUTHORIZED
        );
    }
    fn make_token_with_role(role: &str, exp_offset_seconds: i64) -> String {
        let exp = (chrono::Utc::now().timestamp() + exp_offset_seconds) as usize;
        let claims = json!({
            "sub": "11111111-1111-1111-1111-111111111111",
            "session_id": "22222222-2222-2222-2222-222222222222",
            "email": "user@example.com",
            "role": "",
            "iss": "http://localhost:9999/auth/v1",
            "aud": "authenticated",
            "exp": exp,
            "app_metadata": { "role": role }
        });

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret("test-secret".as_bytes()),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn merchant_only_qua_router_voi_vai_tro_dung_tra_ve_200() {
        let app = build_router(AppState::for_test(4000));
        let token = make_token_with_role("merchant", 3600);

        assert_eq!(
            call_me_path(app, "/v1/merchant-only", &token).await,
            StatusCode::OK
        );
    }

    #[tokio::test]
    async fn merchant_only_qua_router_voi_vai_tro_sai_tra_ve_403() {
        let app = build_router(AppState::for_test(4000));
        let token = make_token_with_role("customer", 3600);

        assert_eq!(
            call_me_path(app, "/v1/merchant-only", &token).await,
            StatusCode::FORBIDDEN
        );
    }
    #[tokio::test]
    async fn me_qua_router_voi_session_da_bi_revoke_tra_ve_401() {
        let state = AppState::for_test(4000);
        let session_id = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
        state
            .revocation
            .revoke_session(session_id, 60)
            .await
            .unwrap();

        let app = build_router(state);
        let token = make_token("test-secret", 3600);

        assert_eq!(
            call_me_path(app, "/v1/me", &token).await,
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn me_qua_router_voi_user_da_bi_revoke_tra_ve_401() {
        let state = AppState::for_test(4000);
        let user_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        state.revocation.revoke_user(user_id, 60).await.unwrap();

        let app = build_router(state);
        let token = make_token("test-secret", 3600);

        assert_eq!(
            call_me_path(app, "/v1/me", &token).await,
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn logout_qua_router_sau_do_me_bi_tu_choi_vi_da_revoke() {
        let app = build_router(AppState::for_test(4000));
        let token = make_token("test-secret", 3600);

        let logout_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/auth/logout")
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(logout_response.status(), StatusCode::NO_CONTENT);

        assert_eq!(
            call_me_path(app, "/v1/me", &token).await,
            StatusCode::UNAUTHORIZED
        );
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use axum::body::Body;
//     use axum::http::{Request, StatusCode};

//     use tower::ServiceExt;

//     #[tokio::test]
//     async fn request_qua_full_router_co_tracelayer_van_tra_200() {
//         let state = AppState::for_test(4000);
//         let app = build_router(state);

//         let response = app
//             .oneshot(
//                 Request::builder()
//                     .uri("/health")
//                     .body(Body::empty())
//                     .unwrap(),
//             )
//             .await
//             .unwrap();

//         assert_eq!(response.status(), StatusCode::OK);
//     }

//     #[tokio::test]
//     async fn echo_qua_router_tra_ve_400_khi_qua_dai() {
//         let state = AppState::for_test(4000);
//         let app = build_router(state);

//         let text_qua_dai = "a".repeat(51);
//         let response = app
//             .oneshot(
//                 Request::builder()
//                     .uri(format!("/echo/{text_qua_dai}"))
//                     .body(Body::empty())
//                     .unwrap(),
//             )
//             .await
//             .unwrap();

//         assert_eq!(response.status(), StatusCode::BAD_REQUEST);
//     }
//     #[tokio::test]
//     async fn me_qua_router_voi_token_hop_le_tra_ve_200() {
//         use axum::http::header::AUTHORIZATION;
//         use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
//         use serde_json::json;

//         let state = AppState::for_test(4000);
//         let app = build_router(state);

//         let exp = (chrono::Utc::now().timestamp() + 3600) as usize;
//         let claims = json!({
//             "sub": "11111111-1111-1111-1111-111111111111",
//             "session_id": "22222222-2222-2222-2222-222222222222",
//             "email": "user@example.com",
//             "role": "",
//             "iss": "http://localhost:9999/auth/v1",
//             "aud": "authenticated",
//             "exp": exp,
//             "app_metadata": {}
//         });
//         let token = encode(
//             &Header::new(Algorithm::HS256),
//             &claims,
//             &EncodingKey::from_secret("test-secret".as_bytes()),
//         )
//         .unwrap();

//         let response = app
//             .oneshot(
//                 Request::builder()
//                     .uri("/v1/me")
//                     .header(AUTHORIZATION, format!("Bearer {token}"))
//                     .body(Body::empty())
//                     .unwrap(),
//             )
//             .await
//             .unwrap();

//         assert_eq!(response.status(), StatusCode::OK);
//     }
// }
