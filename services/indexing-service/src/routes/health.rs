//! Health Check Endpoint
//!
//! Provides a simple endpoint to verify that the **Indexing Service** is
//! operational.
//!
//! **GET /status**
//! → Returns `{"service": "indexing-service", "status": "running"}`
use crate::models::responses::HealthResponse;
use axum::response::Json;

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "indexing-service".to_string(),
        status: "running".to_string(),
    })
}