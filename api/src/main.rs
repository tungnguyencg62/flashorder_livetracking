use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello_world));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Server đang chạy tại http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}
async fn hello_world() -> &'static str {
    "Hello, FlashOrder!"
}
