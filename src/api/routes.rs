use actix_web::{web, HttpResponse, Responder};
use crate::api::controllers;

/// Configure API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // Command execution
            .service(
                web::resource("/execute/show")
                    .route(web::post().to(controllers::execute_show_command)),
            )
            .service(
                web::resource("/execute/configure")
                    .route(web::post().to(controllers::execute_config_commands)),
            )
            // Interface management
            .service(
                web::resource("/interfaces/configure")
                    .route(web::post().to(controllers::configure_interface)),
            ),
    )
    .service(
        web::resource("/health")
            .route(web::get().to(health_check)),
    );
}

/// Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Healthy")
}