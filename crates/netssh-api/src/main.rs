use actix_web::{web, App, HttpServer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use netssh_api::routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    info!("Starting Netssh-rs API server");

    // Start API server
    info!("Starting API server on 127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .configure(routes::configure)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
} 