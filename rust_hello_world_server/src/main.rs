use actix_web::{get, App, HttpServer, Responder};
use std::time::Duration;
use tokio::time::sleep;

#[get("/")]
async fn index() -> impl Responder {
    // Introduce a 100 microsecond delay
    sleep(Duration::from_micros(10)).await;
    
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server on port 8080");
    HttpServer::new(|| {
        App::new()
            .service(index)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
