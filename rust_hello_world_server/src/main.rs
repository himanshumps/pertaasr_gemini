use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Define a simple structure for the response
    #[derive(Serialize)]
    struct Message {
        message: String,
    }

    // A handler that returns a JSON message
    let hello_handler = || async {
        Json(Message {
            message: "Hello, World! from Rust REST server".to_string(),
        })
    };

    // Build the application router
    let app = Router::new().route("/hello", get(hello_handler));

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}