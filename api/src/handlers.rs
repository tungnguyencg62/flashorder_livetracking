use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::AppError::{self, Auth},
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

#[derive(Serialize)]
pub struct MeResponse {
    id: Uuid,
    email: Option<String>,
}

pub async fn me(AuthUser(claims): AuthUser) -> Json<MeResponse> {
    Json(MeResponse {
        id: claims.sub,
        email: claims.email,
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

    #[tokio::test]
    async fn me_tra_ve_dung_thong_tin_tu_claims() {
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

        let Json(response) = me(AuthUser(claims)).await;

        assert_eq!(response.id, expected_id);
        assert_eq!(response.email.as_deref(), Some("user@example.com"));
    }
}
