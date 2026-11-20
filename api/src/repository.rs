use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize)]
pub struct Profile {
    pub id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn insert_profile(
    pool: &PgPool,
    id: Uuid,
    email: String,
    full_name: Option<String>,
) -> Result<Profile, sqlx::Error> {
    sqlx::query_as::<_, Profile>(
        "INSERT INTO profiles (id, email, full_name) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(id)
    .bind(email)
    .bind(full_name)
    .fetch_one(pool)
    .await
}

pub async fn find_profile_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Profile>, sqlx::Error> {
    sqlx::query_as::<_, Profile>("SELECT * FROM profiles WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    async fn pool_tu_database_url() -> PgPool {
        dotenvy::dotenv().ok();
        let config = Config::from_env().expect("cần DATABASE_URL để chạy test này");
        PgPool::connect(&config.database_url)
            .await
            .expect("không kết nối được Postgres — nhớ `docker compose up -d db`")
    }

    #[tokio::test]
    async fn insert_roi_tim_lai_dung_profile() {
        let pool = pool_tu_database_url().await;
        let id = Uuid::new_v4();

        let inserted = insert_profile(
            &pool,
            id,
            "test@example.com".to_string(),
            Some("Test User".to_string()),
        )
        .await
        .expect("insert phải thành công");

        assert_eq!(inserted.email, "test@example.com");
        assert_eq!(inserted.full_name.as_deref(), Some("Test User"));

        let found = find_profile_by_id(&pool, id)
            .await
            .expect("query phải thành công")
            .expect("phải tìm thấy profile vừa insert");

        assert_eq!(found.id, id);

        // Dọn dẹp thủ công — đây chính là nhược điểm sẽ nói ở phần giải thích,
        // và sẽ được giải quyết triệt để ở bước 13 bằng #[sqlx::test].
        sqlx::query("DELETE FROM profiles WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .ok();
    }
}
