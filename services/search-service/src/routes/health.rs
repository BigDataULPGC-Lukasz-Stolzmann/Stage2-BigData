//! Health Check Endpoint
//!
//! Simple route to verify that the **Search Service** is up and running.
//!
//! **GET /status**
//! → Returns `{"service":"search-service","status":"running"}`

use crate::models::responses::HealthResponse;
use axum::response::Json;


/// Returns the current health status of the Search Service.
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "search-service".to_string(),
        status: "running".to_string(),
    })
}