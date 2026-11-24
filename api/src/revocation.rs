use async_trait::async_trait;
use redis::aio::ConnectionManager;
use uuid::Uuid;

use crate::error::AppError;

#[async_trait]
pub trait RevocationStore: Send + Sync {
    async fn revoke_session(&self, session_id: Uuid, ttl_seconds: u64) -> Result<(), AppError>;
    async fn revoke_user(&self, user_id: Uuid, ttl_seconds: u64) -> Result<(), AppError>;
    async fn is_session_revoked(&self, session_id: Uuid) -> Result<bool, AppError>;
    async fn is_user_revoked(&self, user_id: Uuid) -> Result<bool, AppError>;
}

pub struct RedisRevocationStore {
    conn: ConnectionManager,
}

impl RedisRevocationStore {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl RevocationStore for RedisRevocationStore {
    async fn revoke_session(&self, session_id: Uuid, ttl_seconds: u64) -> Result<(), AppError> {
        let mut conn = self.conn.clone();

        redis::cmd("SET")
            .arg(format!("revoked:session:{session_id}"))
            .arg("1")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|err| AppError::Cache(err.to_string()))
    }

    async fn revoke_user(&self, user_id: Uuid, ttl_seconds: u64) -> Result<(), AppError> {
        let mut conn = self.conn.clone();

        redis::cmd("SET")
            .arg(format!("revoked:user:{user_id}"))
            .arg("1")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|err| AppError::Cache(err.to_string()))
    }
    async fn is_session_revoked(&self, session_id: Uuid) -> Result<bool, AppError> {
        let mut conn = self.conn.clone();
        redis::cmd("EXISTS")
            .arg(format!("revoked:session:{session_id}"))
            .query_async(&mut conn)
            .await
            .map_err(|err| AppError::Cache(err.to_string()))
    }
    async fn is_user_revoked(&self, user_id: Uuid) -> Result<bool, AppError> {
        let mut conn = self.conn.clone();
        redis::cmd("EXISTS")
            .arg(format!("revoked:user:{user_id}"))
            .query_async(&mut conn)
            .await
            .map_err(|err| AppError::Cache(err.to_string()))
    }
}
#[cfg(test)]
pub struct InMemoryRevocationStore {
    sessions: std::sync::Mutex<std::collections::HashSet<Uuid>>,
    users: std::sync::Mutex<std::collections::HashSet<Uuid>>,
}

#[cfg(test)]
impl InMemoryRevocationStore {
    pub fn new() -> Self {
        Self {
            sessions: std::sync::Mutex::new(std::collections::HashSet::new()),
            users: std::sync::Mutex::new(std::collections::HashSet::new()),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl RevocationStore for InMemoryRevocationStore {
    async fn revoke_session(&self, session_id: Uuid, _ttl_seconds: u64) -> Result<(), AppError> {
        self.sessions.lock().unwrap().insert(session_id);
        Ok(())
    }

    async fn revoke_user(&self, user_id: Uuid, _ttl_seconds: u64) -> Result<(), AppError> {
        self.users.lock().unwrap().insert(user_id);
        Ok(())
    }

    async fn is_session_revoked(&self, session_id: Uuid) -> Result<bool, AppError> {
        Ok(self.sessions.lock().unwrap().contains(&session_id))
    }

    async fn is_user_revoked(&self, user_id: Uuid) -> Result<bool, AppError> {
        Ok(self.users.lock().unwrap().contains(&user_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_store() -> RedisRevocationStore {
        let redis_url = std::env::var("REDIS_URL")
            .expect("cần REDIS_URL — nhớ source .env và docker compose up -d redis");
        let conn = crate::cache::connect(&redis_url)
            .await
            .expect("không kết nối được Redis");
        RedisRevocationStore::new(conn)
    }

    async fn cleanup(key: &str) {
        let redis_url = std::env::var("REDIS_URL").unwrap();
        let mut conn = crate::cache::connect(&redis_url).await.unwrap();
        let _: () = redis::cmd("DEL")
            .arg(key)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn session_chua_revoke_tra_ve_false() {
        let store = test_store().await;
        let session_id = Uuid::new_v4();

        assert!(!store.is_session_revoked(session_id).await.unwrap());
    }

    #[tokio::test]
    async fn revoke_session_roi_kiem_tra_tra_ve_true() {
        let store = test_store().await;
        let session_id = Uuid::new_v4();

        store.revoke_session(session_id, 60).await.unwrap();
        assert!(store.is_session_revoked(session_id).await.unwrap());

        cleanup(&format!("revoked:session:{session_id}")).await;
    }

    #[tokio::test]
    async fn revoke_user_roi_kiem_tra_tra_ve_true() {
        let store = test_store().await;
        let user_id = Uuid::new_v4();

        store.revoke_user(user_id, 60).await.unwrap();
        assert!(store.is_user_revoked(user_id).await.unwrap());

        cleanup(&format!("revoked:user:{user_id}")).await;
    }
}
