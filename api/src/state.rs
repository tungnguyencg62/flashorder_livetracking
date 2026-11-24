use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
}

#[cfg(test)]
impl AppState {
    /// Dựng AppState cho test KHÔNG cần Postgres thật đang chạy.
    /// `db` chỉ là pool "lazy" (connect_lazy không mở kết nối thật ngay) —
    /// chỉ dùng được cho test không thực sự chạy query qua state.db
    /// (ví dụ info, echo). Test cần DB thật phải dùng #[sqlx::test] (bước 13).
    pub fn for_test(port: u16) -> Self {
        // AppState {
        //     config: Arc::new(Config {
        //         port,
        //         database_url: "postgres://test".to_string(),
        //         jwt_secret: "test-secret".to_string(),
        //         jwt_issuer: "http://localhost:9999/auth/v1".to_string(),
        //         jwt_audience: "authenticated".to_string(),
        //     }),
        //     db: sqlx::postgres::PgPoolOptions::new()
        //         .connect_lazy("postgres://test")
        //         .expect("connect_lazy không lỗi vì chưa thực sự mở kết nối"),
        // }
        let db = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://test")
            .expect("connect_lazy không lỗi vì chưa thực sự mở kết nối");
        Self::for_test_with_db(port, db)
    }
    pub fn for_test_with_db(port: u16, db: PgPool) -> Self {
        AppState {
            config: Arc::new(Config {
                port,
                database_url: "postgres://test".to_string(),
                jwt_secret: "test-secret".to_string(),
                jwt_issuer: "http://localhost:9999/auth/v1".to_string(),
                jwt_audience: "authenticated".to_string(),
            }),
            db,
        }
    }
}
