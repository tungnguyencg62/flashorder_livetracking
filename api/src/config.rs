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

    #[test]
    fn thieu_database_url_tra_ve_loi_config() {
        unsafe { std::env::remove_var("DATABASE_URL") };
        unsafe { std::env::remove_var("PORT") };

        let result = Config::from_env();

        match result {
            Err(AppError::Config(msg)) => assert!(msg.contains("DATABASE_URL")),
            Err(other) => panic!("kỳ vọng Err(AppError::Config(..)), nhận: {other:?}"),
            Ok(_) => panic!("kỳ vọng Err(AppError::Config(..)), nhưng lại nhận Ok(Config)"),
        }
    }
}
