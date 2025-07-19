pub mod api;
pub mod auth;
pub mod storage;
pub mod sync;
pub mod webdav;
pub mod caldav;
pub mod monitoring;
pub mod plugins;
pub mod config;

use actix_web::{web, App, HttpServer, middleware};
use tracing_actix_web::TracingLogger;
use crate::config::AppConfig;
use crate::config::database::DatabasePool;
use crate::plugins::manager::PluginManager;
use crate::plugins::auth_local::LocalAuthPlugin;
use crate::plugins::monitoring_prometheus::PrometheusMonitoringPlugin;
use crate::plugins::storage_local::LocalStoragePlugin;
use std::sync::Arc;

pub async fn run_server(config: AppConfig) -> std::io::Result<()> {
    // Initialize database
    let db_pool = DatabasePool::new(&config.database).await
        .expect("Failed to connect to database");
    
    // Run migrations
    db_pool.migrate().await
        .expect("Failed to run database migrations");
    
    // Initialize storage
    let storage = storage::create_storage_backend(&config.storage).await
        .expect("Failed to initialize storage backend");
    
    // Initialize plugin manager
    let plugin_manager = Arc::new(PluginManager::new());
    // Register built-in plugins
    plugin_manager.register_auth_plugin(Arc::new(LocalAuthPlugin));
    plugin_manager.register_monitoring_plugin(Arc::new(PrometheusMonitoringPlugin));
    plugin_manager.register_storage_plugin(Arc::new(LocalStoragePlugin));
    
    // Create application state
    let app_state = web::Data::new(AppState {
        config: config.clone(),
        db_pool,
        storage: Arc::from(storage),
        plugin_manager,
    });
    
    tracing::info!("Starting server on {}:{}", config.server.host, config.server.port);
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(TracingLogger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(
                actix_cors::Cors::default()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost") ||
                        origin.as_bytes().starts_with(b"https://localhost")
                    })
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS", "PROPFIND", "PROPPATCH"])
                    .allowed_headers(vec!["authorization", "accept", "content-type", "x-requested-with"])
                    .supports_credentials()
            )
            .configure(api::configure_routes)
            .configure(webdav::configure)
            .configure(caldav::configure)
            .service(actix_files::Files::new("/static", "./static").index_file("index.html"))
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db_pool: DatabasePool,
    pub storage: Arc<dyn storage::StorageBackend>,
    pub plugin_manager: Arc<PluginManager>,
}
