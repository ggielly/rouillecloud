//! API module for REST endpoints

pub mod files;
pub mod auth;
pub mod websocket;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(files::configure)
            .configure(auth::configure)
            .configure(websocket::configure)
    );
}
