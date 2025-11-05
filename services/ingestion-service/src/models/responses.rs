//! Ingestion Service — Response Models
//!
//! Defines the **API response structures** used by the Ingestion Service.
//!
//! ## Structures
//! - `HealthResponse` — used by `/status` for service health reporting  
//! - `IngestResponse` — returned after successful ingestion of a book  
//! - `StatusResponse` — reports processing status for a specific book  
//! - `ListResponse` — lists all available ingested book IDs

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct HealthResponse {
    pub service: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IngestResponse {
    pub book_id: u32,
    pub status: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub book_id: u32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse {
    pub count: usize,
    pub books: Vec<u32>,
}
