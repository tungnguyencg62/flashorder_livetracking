use axum::{
    Json,
    extract::{Path, State},
};

use reqwest::{StatusCode, header::AUTHORIZATION};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    auth::{Admin, AuthUser, Merchant, RequireRole},
    error::AppError::{self},
    repository::{self, Profile},
    revocation,
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
    AuthUser(claims, _): AuthUser,
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
    RequireRole(AuthUser(claims, _), _): RequireRole<Merchant>,
) -> Json<MerchantOnlyResponse> {
    Json(MerchantOnlyResponse {
        message: format!("Xin chào, merchant! User ID của bạn là {}", claims.sub),
    })
}

pub async fn logout(
    AuthUser(claims, token): AuthUser,
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, AppError> {
    let ttl_seconds = (claims.exp as i64 - chrono::Utc::now().timestamp()).max(1) as u64;

    state
        .revocation
        .revoke_session(claims.session_id, ttl_seconds)
        .await
        .map_err(|_| AppError::Cache("không ghi được revocation, logout thất bại".to_string()))?;

    let client = reqwest::Client::new();
    let gotrue_logout_url = format!("{}/logout", state.config.gotrue_url);

    match client
        .post(&gotrue_logout_url)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {}
        Ok(response) => tracing::warn!(
            "GoTrue logout trả về status {} — revocation nội bộ đã ghi nên vẫn coi là logout thành công",
            response.status()
        ),
        Err(err) => tracing::warn!(
            "Không gọi được GoTrue logout ({err}) — revocation nội bộ đã ghi nên vẫn coi là logout thành công"
        ),
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn ban_user(
    RequireRole(AuthUser(_admin, _), _): RequireRole<Admin>,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .revocation
        .revoke_user(user_id, revocation::PERMANENT_BAN_TTL_SECONDS)
        .await
        .map_err(|_| AppError::Cache("không ghi được revocation, ban thất bại".to_string()))?;
    match crate::auth::mint_service_token(&state.config.jwt_secret, &state.config.jwt_audience) {
        Ok(service_token) => {
            let client = reqwest::Client::new();
            let gotrue_ban_url = format!("{}/admin/users/{user_id}", state.config.gotrue_url);
            match client
                .put(&gotrue_ban_url)
                .header(AUTHORIZATION, format!("Bearer {service_token}"))
                .json(&serde_json::json!({ "ban_duration": "87600h" }))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {}
                Ok(response) => tracing::warn!(
                    "GoTrue admin ban trả về status {} — revocation nội bộ đã ghi nên vẫn coi là ban thành công",
                    response.status()
                ),
                Err(err) => tracing::warn!(
                    "Không gọi được GoTrue admin ban ({err}) — revocation nội bộ đã ghi nên vẫn coi là ban thành công"
                ),
            }
        }
        Err(err) => tracing::warn!("Không mint được service token để gọi GoTrue admin ban: {err}"),
    }

    Ok(StatusCode::NO_CONTENT)
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

    use sqlx::PgPool;

    use uuid::Uuid;

    #[sqlx::test]
    async fn me_upsert_profile_lan_dau_tra_ve_profile_moi(pool: PgPool) -> sqlx::Result<()> {
        // let state = AppState {
        //     config: Arc::new(Config {
        //         port: 4000,
        //         database_url: String::new(),
        //         jwt_secret: "test-secret".to_string(),
        //         jwt_issuer: "http://localhost:9999/auth/v1".to_string(),
        //         jwt_audience: "authenticated".to_string(),
        //     }),
        //     db: pool,
        //     revocation: Arc::new(crate::revocation::InMemoryRevocationStore::new()),
        // };
        let state = AppState::for_test_with_db(4000, pool);
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

        let Json(profile) = me(AuthUser(claims, "token-gia".to_string()), State(state))
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

        let result = me(AuthUser(claims, "token-gia".to_string()), State(state)).await;

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

        let Json(response) = merchant_only_demo(RequireRole(
            AuthUser(claims, "token-gia".to_string()),
            PhantomData,
        ))
        .await;

        assert!(response.message.contains(&expected_id.to_string()));
    }
    #[tokio::test]
    async fn ban_user_ghi_revocation_du_gotrue_khong_goi_duoc() {
        let state = AppState::for_test(4000);
        let admin_claims = Claims {
            sub: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            email: Some("admin@example.com".to_string()),
            role: String::new(),
            aud: "authenticated".to_string(),
            exp: 9999999999,
            app_metadata: serde_json::json!({ "role": "admin" }),
        };
        let target_user_id = Uuid::new_v4();

        let result = ban_user(
            RequireRole(AuthUser(admin_claims, "token".to_string()), PhantomData),
            State(state.clone()),
            Path(target_user_id),
        )
        .await;

        assert!(result.is_ok());
        assert!(
            state
                .revocation
                .is_user_revoked(target_user_id)
                .await
                .unwrap()
        );
    }
}
