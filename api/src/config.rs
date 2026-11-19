use std::env;

use crate::error::AppError;

pub struct Config {
    pub port: u16,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        let port = match env::var("PORT").ok() {
            Some(value) => value.parse::<u16>().map_err(|_| {
                AppError::Config(format!("PORT='{value}' không phải số hợp lệ (1-65535)"))
            })?,
            None => 3000,
        };
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("thiếu biến môi trường DATABASE_URL".to_string()))?;
        Ok(Config {
            port: port,
            database_url: database_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Khóa đồng bộ vì `cargo test` chạy các test song song trên nhiều thread,
    // trong khi biến môi trường dùng chung cho toàn bộ process.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn set_env(key: &str, val: &str) {
        // Rust Edition 2024 yêu cầu `unsafe` khi thay đổi biến môi trường của process
        unsafe { env::set_var(key, val) };
    }

    fn remove_env(key: &str) {
        unsafe { env::remove_var(key) };
    }

    #[test]
    fn from_env_uses_default_port_when_unset() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        remove_env("PORT");
        set_env("DATABASE_URL", "postgres://localhost/test_db");

        let cfg = Config::from_env().expect("phải load config thành công");

        assert_eq!(cfg.port, 3000);
        assert_eq!(cfg.database_url, "postgres://localhost/test_db");
    }

    #[test]
    fn from_env_parses_valid_custom_port() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_env("PORT", "8080");
        set_env("DATABASE_URL", "postgres://localhost/test_db");

        let cfg = Config::from_env().expect("phải load config thành công");

        assert_eq!(cfg.port, 8080);
    }

    #[test]
    fn from_env_rejects_non_numeric_port() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_env("PORT", "khong-phai-so");
        set_env("DATABASE_URL", "postgres://localhost/test_db");

        match Config::from_env() {
            Err(AppError::Config(msg)) => {
                assert!(msg.contains("PORT='khong-phai-so' không phải số hợp lệ"));
            }
            Err(other) => panic!("mong đợi AppError::Config, nhận được: {other:?}"),
            Ok(_) => panic!("mong đợi lỗi khi PORT không phải số"),
        }
    }

    #[test]
    fn from_env_rejects_port_out_of_u16_range() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_env("PORT", "70000"); // vượt quá u16::MAX (65535)
        set_env("DATABASE_URL", "postgres://localhost/test_db");

        match Config::from_env() {
            Err(AppError::Config(msg)) => {
                assert!(msg.contains("70000"));
            }
            _ => panic!("mong đợi AppError::Config khi PORT > 65535"),
        }
    }

    #[test]
    fn from_env_fails_when_database_url_missing() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_env("PORT", "3000");
        remove_env("DATABASE_URL");

        match Config::from_env() {
            Err(AppError::Config(msg)) => {
                assert_eq!(msg, "thiếu biến môi trường DATABASE_URL");
            }
            Err(other) => panic!("mong đợi AppError::Config, nhận được: {other:?}"),
            Ok(_) => panic!("mong đợi lỗi khi thiếu DATABASE_URL"),
        }
    }
}
