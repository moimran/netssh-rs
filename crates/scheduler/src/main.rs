use std::sync::Arc;
use std::str::FromStr;

use apalis::prelude::*;
use apalis_sql::sqlite::SqliteStorage as ApalisSqliteStorage;
use axum::Router;
use scheduler::{
    api::create_api_routes,
    board::BoardService,
    config::Config,
    jobs::enhanced_ssh_job_handler_global,
    logging,
    scheduler::JobScheduler,
    storage::{SqliteStorage, Storage},
};
use sqlx::SqlitePool;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration first to get logging settings
    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}. Using defaults.", e);
        Config::default()
    });

    // Initialize logging with configuration
    if let Err(e) = logging::init_logging(config.logging()) {
        eprintln!("Failed to initialize logging: {}", e);
        // Fall back to basic logging
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    info!("Starting Job Scheduler application");
    info!("Configuration loaded: {:?}", config);

    // Set up storage
    let storage = Arc::new(
        SqliteStorage::new(&config.database().url)
            .await
            .expect("Failed to initialize storage"),
    );

    info!("Storage initialized");

    // Initialize storage
    storage
        .initialize()
        .await
        .expect("Failed to initialize storage");

    // Set up Apalis SQLite storage for job queue (separate database to avoid table conflicts)
    let apalis_db_url = config
        .database()
        .url
        .replace("scheduler.db", "apalis_queue.db");
    info!("Connecting to Apalis queue database: {}", apalis_db_url);

    // Use SqliteConnectOptions to create the Apalis database if it doesn't exist
    let apalis_connect_options = sqlx::sqlite::SqliteConnectOptions::from_str(&apalis_db_url)
        .expect("Invalid Apalis database URL")
        .create_if_missing(true);

    let sqlite_pool = SqlitePool::connect_with(apalis_connect_options)
        .await
        .expect("Failed to connect to SQLite for Apalis");

    // Setup Apalis storage (run migrations to create job queue tables)
    info!("Setting up Apalis storage and running migrations...");
    ApalisSqliteStorage::setup(&sqlite_pool)
        .await
        .expect("Failed to setup Apalis storage");
    info!("Apalis storage setup completed");

    let apalis_storage = ApalisSqliteStorage::new(sqlite_pool);

    // Set up job scheduler service
    let job_scheduler = Arc::new(
        JobScheduler::new(
            storage.clone(),
            Arc::new(apalis_storage.clone()),
            config.scheduler().clone(),
        )
        .expect("Failed to create job scheduler"),
    );

    // Set up board service
    let board_service = BoardService::new(storage.clone(), config.board().clone());

    // Build the worker - use enhanced handler by default for Phase 1 implementation
    info!(
        "Building enhanced worker with connection reuse enabled - concurrency: {}, max_idle_time: {}s, max_connections_per_worker: {}",
        config.worker().concurrency,
        config.worker().max_idle_time_seconds,
        config.worker().max_connections_per_worker
    );

    let worker = WorkerBuilder::new("enhanced-ssh-job-worker")
        .concurrency(config.worker().concurrency)
        .data(storage.clone() as Arc<dyn Storage>)
        .backend(apalis_storage.clone())
        .build_fn(enhanced_ssh_job_handler_global);

    info!(
        "Worker configured with concurrency: {}, connection_reuse: {}, failure_strategy: {}",
        config.worker().concurrency,
        config.worker().connection_reuse,
        config.worker().failure_strategy
    );

    // Build the web application
    let app = Router::new()
        .nest("/api", create_api_routes())
        .nest("/board", board_service.routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(axum::Extension(apalis_storage.clone()))
                .layer(axum::Extension(job_scheduler.clone())),
        )
        .with_state(storage);

    info!("Web server configured");

    // Start both the worker and web server
    let bind_addr = config.bind_address();
    info!("Starting server on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    tokio::select! {
        result = async {
            info!("Starting Apalis worker...");
            info!("Worker will process jobs from the queue with concurrency: {}", config.worker().concurrency);
            worker.run().await;
            info!("Apalis worker has stopped");
            Ok::<(), Box<dyn std::error::Error>>(())
        } => {
            if let Err(e) = result {
                error!("Worker error: {}", e);
                eprintln!("Worker error: {}", e);
            } else {
                info!("Worker completed successfully");
            }
        }
        result = async {
            info!("Starting job scheduler...");
            job_scheduler.start().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        } => {
            if let Err(e) = result {
                error!("Job scheduler error: {}", e);
                eprintln!("Job scheduler error: {}", e);
            } else {
                info!("Job scheduler completed successfully");
            }
        }
        result = async {
            info!("Starting web server...");
            axum::serve(listener, app).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        } => {
            if let Err(e) = result {
                error!("Server error: {}", e);
                eprintln!("Server error: {}", e);
            } else {
                info!("Server completed successfully");
            }
        }
    }

    Ok(())
}
