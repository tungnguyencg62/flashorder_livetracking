use axum::{
    Json,
    extract::{Path, State},
};
use chrono::Weekday::Sat;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    auth::{AuthUser, Claims, Merchant, RequireRole},
    error::AppError::{self},
    repository::{self, Profile},
    state::AppState,
};

pub async fn hello_world() -> &'static str {
    "Hello, FlashOrder!"
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
pub async fn ready(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    sqlx::query("SELECT 1").execute(&state.db).await?;

    Ok(Json(HealthResponse { status: "ready" }))
}

#[derive(Serialize)]
pub struct InfoResponse {
    port: u16,
}

pub async fn info(State(state): State<AppState>) -> Json<InfoResponse> {
    Json(InfoResponse {
        port: state.config.port,
    })
}

#[derive(Serialize)]
pub struct EchoResponse {
    message: String,
}
pub async fn echo(Path(text): Path<String>) -> Result<Json<EchoResponse>, AppError> {
    if text.is_empty() || text.chars().count() > 50 {
        return Err(AppError::Validation("Text không được để trống".to_string()));
    }
    Ok(Json(EchoResponse { message: text }))
}

pub async fn me(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Profile>, AppError> {
    let email = claims
        .email
        .ok_or_else(|| AppError::Auth("token không có email".to_string()))?;

    let profile = repository::upsert_profile(&state.db, claims.sub, email).await?;

    Ok(Json(profile))
}

#[derive(Serialize)]
pub struct MerchantOnlyResponse {
    message: String,
}

pub async fn merchant_only_demo(
    RequireRole(AuthUser(claims), _): RequireRole<Merchant>,
) -> Json<MerchantOnlyResponse> {
    Json(MerchantOnlyResponse {
        message: format!("Xin chào, merchant! User ID của bạn là {}", claims.sub),
    })
}

#[cfg(test)]
mod tests {
    use crate::auth::Claims;

    use super::*;

    #[tokio::test]
    async fn info_tra_ve_dung_port_tu_state() {
        let state = AppState::for_test(4000);
        let Json(response) = info(State(state)).await;
        assert_eq!(response.port, 4000);
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

    use crate::config::Config;
    use sqlx::PgPool;
    use std::sync::Arc;

    #[sqlx::test]
    async fn me_upsert_profile_lan_dau_tra_ve_profile_moi(pool: PgPool) -> sqlx::Result<()> {
        let state = AppState {
            config: Arc::new(Config {
                port: 4000,
                database_url: String::new(),
                jwt_secret: "test-secret".to_string(),
                jwt_issuer: "http://localhost:9999/auth/v1".to_string(),
                jwt_audience: "authenticated".to_string(),
            }),
            db: pool,
        };

        let claims = Claims {
            sub: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            email: Some("user@example.com".to_string()),
            role: String::new(),
            aud: "authenticated".to_string(),
            exp: 9999999999,
            app_metadata: serde_json::Value::Null,
        };
        let expected_id = claims.sub;

        let Json(profile) = me(AuthUser(claims), State(state))
            .await
            .expect("me phai thanh cong");

        assert_eq!(profile.id, expected_id);
        assert_eq!(profile.email, "user@example.com");

        Ok(())
    }

    #[tokio::test]
    async fn me_thieu_email_trong_token_tra_ve_loi_auth() {
        let state = AppState::for_test(4000);
        let claims = Claims {
            sub: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            email: None,
            role: String::new(),
            aud: "authenticated".to_string(),
            exp: 9999999999,
            app_metadata: serde_json::Value::Null,
        };

        let result = me(AuthUser(claims), State(state)).await;

        assert!(matches!(result, Err(AppError::Auth(_))));
    }

    use std::marker::PhantomData;

    #[tokio::test]
    async fn merchant_only_demo_tra_ve_dung_message() {
        let claims = Claims {
            sub: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            email: Some("merchant@example.com".to_string()),
            role: String::new(),
            aud: "authenticated".to_string(),
            exp: 9999999999,
            app_metadata: serde_json::json!({ "role": "merchant" }),
        };
        let expected_id = claims.sub;

        let Json(response) = merchant_only_demo(RequireRole(AuthUser(claims), PhantomData)).await;

        assert!(response.message.contains(&expected_id.to_string()));
    }
}
