use std::marker::PhantomData;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub session_id: Uuid,
    pub email: Option<String>,
    pub role: String,
    pub aud: String,
    pub exp: usize,
    #[serde(default)]
    pub app_metadata: Value,
}

impl Claims {
    pub fn business_role(&self) -> &str {
        self.app_metadata
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("customer")
    }
}

pub fn verify_token(
    token: &str,
    secret: &str,
    issuer: &str,
    audience: &str,
) -> Result<Claims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[audience]);
    validation.set_required_spec_claims(&["exp", "iss", "aud"]);

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|err| AppError::Auth(format!("Token không hợp lệ: {err}")))?;

    Ok(data.claims)
}

pub struct AuthUser(pub Claims);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header_value = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Auth("thiếu header Authorization".to_string()))?;
        let token = header_value.strip_prefix("Bearer ").ok_or_else(|| {
            AppError::Auth("header Authorization phải có dạng Bearer <token>".to_string())
        })?;

        let claims = verify_token(
            token,
            &state.config.jwt_secret,
            &state.config.jwt_issuer,
            &state.config.jwt_audience,
        )?;

        Ok(AuthUser(claims))
    }
}

pub trait Role {
    const NAME: &'static str;
}

pub struct Merchant;
impl Role for Merchant {
    const NAME: &'static str = "merchant";
}
pub struct Admin;
impl Role for Admin {
    const NAME: &'static str = "admin";
}

pub struct RequireRole<R: Role>(pub AuthUser, pub PhantomData<R>);

impl<R: Role + Send + Sync> FromRequestParts<AppState> for RequireRole<R> {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;

        let actual_role = user.0.business_role();

        if actual_role != R::NAME {
            return Err(AppError::Forbidden(format!(
                "yêu cầu vai trò '{}', hiện tại là '{}'",
                R::NAME,
                actual_role
            )));
        }
        Ok(RequireRole(user, PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_dung_cac_truong_tu_json_mau_cua_gotrue() {
        let json = r#"{
            "sub": "11111111-1111-1111-1111-111111111111",
            "session_id": "22222222-2222-2222-2222-222222222222",
            "email": "user@example.com",
            "role": "",
            "aud": "authenticated",
            "exp": 9999999999,
            "app_metadata": { "provider": "email" }
        }"#;

        let claims: Claims = serde_json::from_str(json).expect("phải deserialize được");

        assert_eq!(claims.email.as_deref(), Some("user@example.com"));
        assert_eq!(claims.role, "");
        assert_eq!(claims.aud, "authenticated");
    }

    #[test]
    fn thieu_app_metadata_van_deserialize_duoc_nho_serde_default() {
        let json = r#"{
            "sub": "11111111-1111-1111-1111-111111111111",
            "session_id": "22222222-2222-2222-2222-222222222222",
            "email": "user@example.com",
            "role": "",
            "aud": "authenticated",
            "exp": 9999999999
        }"#;

        let claims: Claims =
            serde_json::from_str(json).expect("phải deserialize được dù thiếu app_metadata");

        assert_eq!(claims.app_metadata, Value::Null);
    }
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde_json::json;

    const SECRET: &str = "test-secret";
    const ISSUER: &str = "http://localhost:9999/auth/v1";
    const AUDIENCE: &str = "authenticated";

    fn exp_in(seconds: i64) -> usize {
        (chrono::Utc::now().timestamp() + seconds) as usize
    }

    fn encode_with(alg: Algorithm, secret: &str, claims: &serde_json::Value) -> String {
        encode(
            &Header::new(alg),
            claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn sample_claims(iss: &str, aud: &str, exp: usize) -> serde_json::Value {
        json!({
            "sub": "11111111-1111-1111-1111-111111111111",
            "session_id": "22222222-2222-2222-2222-222222222222",
            "email": "user@example.com",
            "role": "",
            "iss": iss,
            "aud": aud,
            "exp": exp,
            "app_metadata": {}
        })
    }

    #[test]
    fn token_hop_le_duoc_chap_nhan() {
        let claims = sample_claims(ISSUER, AUDIENCE, exp_in(3600));
        let token = encode_with(Algorithm::HS256, SECRET, &claims);

        let result = verify_token(&token, SECRET, ISSUER, AUDIENCE);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().email.as_deref(), Some("user@example.com"));
    }

    #[test]
    fn sai_secret_bi_tu_choi() {
        let claims = sample_claims(ISSUER, AUDIENCE, exp_in(3600));
        let token = encode_with(Algorithm::HS256, "secret-khac", &claims);

        assert!(matches!(
            verify_token(&token, SECRET, ISSUER, AUDIENCE),
            Err(AppError::Auth(_))
        ));
    }

    #[test]
    fn sai_issuer_bi_tu_choi() {
        let claims = sample_claims("https://ke-gia-mao.com", AUDIENCE, exp_in(3600));
        let token = encode_with(Algorithm::HS256, SECRET, &claims);

        assert!(matches!(
            verify_token(&token, SECRET, ISSUER, AUDIENCE),
            Err(AppError::Auth(_))
        ));
    }

    #[test]
    fn sai_audience_bi_tu_choi() {
        let claims = sample_claims(ISSUER, "mot-audience-khac", exp_in(3600));
        let token = encode_with(Algorithm::HS256, SECRET, &claims);

        assert!(matches!(
            verify_token(&token, SECRET, ISSUER, AUDIENCE),
            Err(AppError::Auth(_))
        ));
    }

    #[test]
    fn token_het_han_bi_tu_choi() {
        let claims = sample_claims(ISSUER, AUDIENCE, exp_in(-3600));
        let token = encode_with(Algorithm::HS256, SECRET, &claims);

        assert!(matches!(
            verify_token(&token, SECRET, ISSUER, AUDIENCE),
            Err(AppError::Auth(_))
        ));
    }

    #[test]
    fn sai_thuat_toan_bi_tu_choi() {
        let claims = sample_claims(ISSUER, AUDIENCE, exp_in(3600));
        let token = encode_with(Algorithm::HS384, SECRET, &claims);

        assert!(matches!(
            verify_token(&token, SECRET, ISSUER, AUDIENCE),
            Err(AppError::Auth(_))
        ));
    }
    #[tokio::test]
    async fn token_hop_le_trich_duoc_authuser() {
        let state = AppState::for_test(4000);
        let claims = sample_claims(ISSUER, AUDIENCE, exp_in(3600));
        let token = encode_with(Algorithm::HS256, SECRET, &claims);

        let (mut parts, _) = axum::http::Request::builder()
            .header("Authorization", format!("Bearer {token}"))
            .body(())
            .unwrap()
            .into_parts();

        let result = AuthUser::from_request_parts(&mut parts, &state).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn thieu_header_authorization_bi_tu_choi() {
        let state = AppState::for_test(4000);

        let (mut parts, _) = axum::http::Request::builder()
            .body(())
            .unwrap()
            .into_parts();

        let result = AuthUser::from_request_parts(&mut parts, &state).await;

        assert!(matches!(result, Err(AppError::Auth(_))));
    }

    #[tokio::test]
    async fn header_khong_co_tien_to_bearer_bi_tu_choi() {
        let state = AppState::for_test(4000);

        let (mut parts, _) = axum::http::Request::builder()
            .header("Authorization", "khong-phai-dang-bearer")
            .body(())
            .unwrap()
            .into_parts();

        let result = AuthUser::from_request_parts(&mut parts, &state).await;

        assert!(matches!(result, Err(AppError::Auth(_))));
    }
    #[test]
    fn business_role_doc_tu_app_metadata() {
        let mut claims_json = sample_claims(ISSUER, AUDIENCE, exp_in(3600));
        claims_json["app_metadata"] = json!({ "role": "merchant" });
        let claims: Claims = serde_json::from_value(claims_json).unwrap();

        assert_eq!(claims.business_role(), "merchant");
    }

    #[test]
    fn business_role_mac_dinh_customer_khi_khong_co() {
        let claims: Claims =
            serde_json::from_value(sample_claims(ISSUER, AUDIENCE, exp_in(3600))).unwrap();

        assert_eq!(claims.business_role(), "customer");
    }

    fn claims_with_role(role: &str, exp: usize) -> serde_json::Value {
        let mut claims = sample_claims(ISSUER, AUDIENCE, exp);
        claims["app_metadata"] = json!({ "role": role });
        claims
    }

    #[tokio::test]
    async fn require_role_merchant_cho_qua_dung_vai_tro() {
        let state = AppState::for_test(4000);
        let claims = claims_with_role("merchant", exp_in(3600));
        let token = encode_with(Algorithm::HS256, SECRET, &claims);

        let (mut parts, _) = axum::http::Request::builder()
            .header("Authorization", format!("Bearer {token}"))
            .body(())
            .unwrap()
            .into_parts();

        let result = RequireRole::<Merchant>::from_request_parts(&mut parts, &state).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn require_role_merchant_tu_choi_vai_tro_khac() {
        let state = AppState::for_test(4000);
        let claims = claims_with_role("customer", exp_in(3600));
        let token = encode_with(Algorithm::HS256, SECRET, &claims);

        let (mut parts, _) = axum::http::Request::builder()
            .header("Authorization", format!("Bearer {token}"))
            .body(())
            .unwrap()
            .into_parts();

        let result = RequireRole::<Merchant>::from_request_parts(&mut parts, &state).await;

        assert!(matches!(result, Err(AppError::Forbidden(_))));
    }
}
