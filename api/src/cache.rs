use redis::aio::ConnectionManager;

pub async fn connect(redis_url: &str) -> Result<ConnectionManager, redis::RedisError> {
    let client = redis::Client::open(redis_url)?;

    client.get_connection_manager().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ket_noi_va_ping_thanh_cong() {
        let redis_url = std::env::var("REDIS_URL")
            .expect("cần REDIS_URL — nhớ `source .env` và `docker compose up -d redis`");

        let mut conn = connect(&redis_url).await.expect("không kết nối được Redis");

        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .expect("PING phải thành công");

        assert_eq!(pong, "PONG");
    }
}
