//! Health Check Endpoint
//!
//! Provides a simple endpoint to verify that the **Ingestion Service** is
//! operational.
//!
//! **GET /status**
//! → Returns `{"service": "ingestion-service", "status": "running"}`

use crate::models::responses::HealthResponse;
use axum::response::Json;

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "ingestion-service".to_string(),
        status: "running".to_string(),
    })
}
