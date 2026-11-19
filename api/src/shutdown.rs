use axum::Router;

pub async fn run(
    listener: tokio::net::TcpListener,
    app: Router,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) {
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .unwrap();
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Nhận Ctrl+C, đang tắt server...");
        },
        _ = terminate => {
            tracing::info!("Nhận SIGTERM, đang tắt server...");
        },
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, routes::build_router, state::AppState};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn server_dung_lai_khi_nhan_tin_hieu_shutdown() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let state = AppState {
            config: Arc::new(Config { port: 0 }),
        };
        let app = build_router(state);

        let (tx, rx) = oneshot::channel::<()>();
        let shutdown = async {
            rx.await.ok();
        };

        let server_task = tokio::spawn(run(listener, app, shutdown));

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(
            !server_task.is_finished(),
            "server phải vẫn đang chạy trước khi nhận tín hiệu"
        );

        tx.send(()).unwrap();

        let result = tokio::time::timeout(Duration::from_secs(2), server_task).await;
        assert!(
            result.is_ok(),
            "server phải shutdown trong 2 giây sau khi nhận tín hiệu"
        );
    }
}
