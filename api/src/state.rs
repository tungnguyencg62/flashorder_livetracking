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
        AppState {
            config: Arc::new(Config {
                port,
                database_url: "postgres://test".to_string(),
            }),
            db: sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://test")
                .expect("connect_lazy không lỗi vì chưa thực sự mở kết nối"),
        }
    }
}
