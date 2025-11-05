//! # Response Models for Indexing Service
//!
//! Defines the response data structures returned by the
//! **Indexing Service** API endpoints.
//!
//! ## Included Responses
//! - `HealthResponse` — Reports service health and uptime.
//! - `IndexResponse` — Returned after indexing a single book.
//! - `RebuildResponse` — Summarizes results of a full index rebuild.
//! - `IndexStatusResponse` — Provides current indexing statistics.use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct HealthResponse {
    pub service: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexResponse {
    pub book_id: u32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RebuildResponse {
    pub books_processed: usize,
    pub elapsed_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexStatusResponse {
    pub books_indexed: usize,
    pub last_update: String,
    pub index_size_mb: f64,
}
