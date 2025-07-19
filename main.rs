use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use fileshare_server::api;
use tracing::info;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to FileShare-App Server!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let server_address = "127.0.0.1:8080";
    info!("🚀 Starting server at http://{}", server_address);

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(api::health_check) // Mount the API module
            // TODO: Register other API, WebDAV, and CalDAV routes here
    })
    .bind(server_address)?
    .run()
    .await
}