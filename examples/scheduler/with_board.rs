use std::sync::Arc;

use axum::Router;
use scheduler::{
    api::create_api_routes,
    board::BoardService,
    config::{BoardConfig, Config},
    storage::{SqliteStorage, Storage},
};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Job Scheduler with Board UI example");

    // Create configuration
    let config = Config {
        database: scheduler::config::DatabaseConfig {
            url: "sqlite:example_with_board.db".to_string(),
            max_connections: 10,
        },
        server: scheduler::config::ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
        },
        worker: scheduler::config::WorkerConfig {
            concurrency: 2,
            timeout_seconds: 300,
        },
        board: BoardConfig {
            enabled: true,
            ui_path: "/board".to_string(),
            api_prefix: "/board/api".to_string(),
            auth_enabled: false,
        },
        logging: scheduler::config::LoggingConfig {
            level: "info".to_string(),
            file: None,
            format: None,
            rotation: None,
        },
        scheduler: scheduler::config::SchedulerConfig {
            enabled: true,
            poll_interval_seconds: 30,
            timezone: None,
            max_concurrent_jobs: 10,
        },
    };

    // Set up storage
    let storage = Arc::new(SqliteStorage::new(&config.database.url).await?);

    // Initialize storage
    storage.initialize().await?;

    // Set up board service
    let board_service = BoardService::new(storage.clone(), config.board.clone());

    // Build the web application
    let app = Router::new()
        .nest("/api", create_api_routes())
        .nest("/board", board_service.routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(storage);

    let bind_addr = config.bind_address();
    info!("Starting server with Board UI on http://{}", bind_addr);
    info!("Board UI available at: http://{}/board", bind_addr);
    info!("API available at: http://{}/api", bind_addr);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
