use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::error::AppError;

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
}
