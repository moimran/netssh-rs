use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use netssh_rs::api::ApiServerConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    println!("Starting Netssh-rs REST API example");

    // Start API server
    let config = ApiServerConfig {
        address: "127.0.0.1".to_string(),
        port: 8080,
    };
    
    println!("Starting API server on {}:{}", config.address, config.port);
    println!("API endpoints:");
    println!("  POST   /api/execute/show");
    println!("  POST   /api/execute/configure");
    println!("  POST   /api/interfaces/configure");
    println!("  GET    /health");
    
    println!("\nSee README.md for example usage");
    
    HttpServer::new(move || {
        App::new()
            .configure(netssh_rs::api::routes::configure)
    })
    .bind(format!("{}:{}", config.address, config.port))?
    .run()
    .await
}
