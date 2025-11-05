//! Response Models for Search Service API
//!
//! Defines the JSON response structures returned by the Search Service endpoints.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;


/// Response for the /status health check endpoint.
#[derive(Deserialize, Serialize, Debug)]
pub struct HealthResponse {
    pub service: String,
    pub status: String,
}

/// Represents a single book in search results.
#[derive(Debug, Serialize, Deserialize)]
pub struct BookResult {
    pub book_id: u32,
    pub title: String,
    pub author: String,
    pub language: String,
    pub year: Option<u32>,
}


/// Response for search queries (GET /search endpoint).
///
/// Returns the search query, applied filters, and matching books.
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub filters: HashMap<String, String>,
    pub count: usize,
    pub results: Vec<BookResult>,
}