use actix_web::{get, App, HttpServer, Responder};

// This function is called a "handler" and responds to the / route
#[get("/")]
async fn index() -> impl Responder {
    "Hello world!" // Actix automatically converts this string slice into an HTTP response
}

#[actix_web::main] // Marks the main function as the entry point for Actix's async runtime
async fn main() -> std::io::Result<()> {
    println!("Starting server on port 8080");
    // Create an instance of the HttpServer
    HttpServer::new(|| {
        // Configure the application with routes/services
        App::new()
            .service(index) // Register the `index` handler function
    })
    .bind(("127.0.0.1", 8080))? // Bind the server to localhost:8080
    .run() // Start the server
    .await // Wait for the server to stop
}