use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

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
}
