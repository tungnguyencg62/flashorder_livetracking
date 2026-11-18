use std::env;

pub struct Config {
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let port = match env::var("PORT").ok() {
            Some(value) => value
                .parse::<u16>()
                .map_err(|_| format!("PORT='{value}' không phải số hợp lệ (1-65535)"))?,
            None => 3000,
        };
        Ok(Config { port: port })
    }
}
