//! Indexing Service
//!
//! Handles **indexing and search preparation** for eBooks stored in the datalake.
//! The service reads preprocessed text files, extracts metadata and tokens,
//! and builds a searchable index using either **Redis** or **PostgreSQL** backends.
//!
//! ## Responsibilities
//! - Index new books on demand  
//! - Rebuild the entire index from the datalake  
//! - Provide index statistics and health status  
//! - Support multiple storage backends (Redis or PostgreSQL)
//!
//! ## Environment Variables
//! - `BACKEND_TYPE`: Selects the storage backend (`redis` or `postgres`)  
//! - `REDIS_URL`: Redis connection URL (default: `redis://redis:6379`)  
//! - `DATABASE_URL`: PostgreSQL connection string  
//! - `PORT`: Service port (default: `7002`)

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

mod models;
mod routes;
mod services;
mod utils;

use models::storage::{PostgresBackend, RedisBackend, StorageBackend};
use routes::{
    health::health_check,
    index::{get_index_status, index_book, rebuild_index},
};

type Backend = Arc<dyn StorageBackend + Send + Sync>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("indexing_service=info,tower_http=info")
        .init();

    let backend_type = std::env::var("BACKEND_TYPE").unwrap_or_else(|_| "redis".to_string());
    let backend: Backend = match backend_type.to_lowercase().as_str() {
        "postgres" | "postgresql" => {
            let database_url = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://user:password@postgres_db:5432/datamart_db".to_string());

            info!("Using PostgreSQL backend");
            let postgres_backend = PostgresBackend::new(&database_url).await
                .expect("Failed to connect to PostgreSQL");

            Arc::new(postgres_backend)
        }
        "redis" | _ => {
            let redis_url = std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://redis:6379".to_string());

            info!("Using Redis backend");
            let redis_backend = RedisBackend::new(&redis_url)
                .expect("Failed to connect to Redis");

            Arc::new(redis_backend)
        }
    };

    if let Err(e) = backend.test_connection().await {
        error!("Failed to connect to storage backend: {}", e);
        std::process::exit(1);
    }
    info!("Storage backend connection successful");

    let app = Router::new()
        .route("/status", get(health_check))
        .route("/index/update/:book_id", post(index_book))
        .route("/index/rebuild", post(rebuild_index))
        .route("/index/status", get(get_index_status))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(backend);

    let port = std::env::var("PORT").unwrap_or_else(|_| "7002".to_string());
    let addr = format!("0.0.0.0:{}", port);

    info!("Indexing service starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}