use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;

use crate::{error::AppError, state::AppState};

pub async fn hello_world() -> &'static str {
    "Hello, FlashOrder!"
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[derive(Serialize)]
pub struct InfoResponse {
    port: u16,
}

pub async fn info(State(state): State<AppState>) -> Json<InfoResponse> {
    Json(InfoResponse {
        port: state.config.port,
    })
}

#[derive(Serialize)]
pub struct EchoResponse {
    message: String,
}
pub async fn echo(Path(text): Path<String>) -> Result<Json<EchoResponse>, AppError> {
    if text.is_empty() || text.chars().count() > 50 {
        return Err(AppError::Validation("Text không được để trống".to_string()));
    }
    Ok(Json(EchoResponse { message: text }))
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::config::Config;
//     use std::sync::Arc;

//     #[tokio::test]
//     async fn info_tra_ve_dung_port_tu_state() {
//         let state = AppState {
//             config: Arc::new(Config { port: 4000 }),
//         };
//         let Json(response) = info(State(state)).await;
//         assert_eq!(response.port, 4000);
//     }

//     #[tokio::test]
//     async fn echo_hop_le_tra_ve_ok() {
//         let result = echo(Path("xin chào".to_string())).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn echo_rong_tra_ve_loi_validation() {
//         let result = echo(Path(String::new())).await;
//         match result {
//             Err(AppError::Validation(_)) => {}
//             _ => panic!("phải trả về AppError::Validation khi text rỗng"),
//         }
//     }
// }
