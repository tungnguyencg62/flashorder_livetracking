use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Lỗi cấu hình: {0}")]
    Config(String),

    #[error("Dữ liệu không hợp lệ: {0}")]
    Validation(String),

    #[error("lỗi database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Không xác thực được: {0}")]
    Auth(String),

    #[error("Không đủ quyền: {0}")]
    Forbidden(String),

    #[error("Lỗi cache: {0}")]
    Cache(String),
}

#[derive(serde::Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Database(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Auth(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Cache(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
        (
            status,
            Json(ErrorBody {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loi_database_tra_ve_503() {
        let response = AppError::from(sqlx::Error::RowNotFound).into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
