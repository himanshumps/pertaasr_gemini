//use std::time::Duration;
use actix_web::{get, App, HttpServer, Responder};
//use actix_web::rt::time::{sleep};

//const DURATION: Duration = Duration::from_nanos(10);

#[get("/")]
async fn index() -> impl Responder {
    // Introduce a 100 microsecond delay
    //sleep(DURATION).await;
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server on port 8083");
    HttpServer::new(|| {
        App::new()
            .service(index)
    })
    .bind(("0.0.0.0", 8083))?
    .run()
    .await
}
