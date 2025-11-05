//! Integration Tests for Indexing Service
//!
//! Simplified tests that focus on basic functionality that should always work.

use serde_json::Value;

#[tokio::test]
async fn test_health_check() {
    let response = reqwest::get("http://0.0.0.0:7002/status")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "running");
    assert_eq!(body["service"], "indexing-service");
}

#[tokio::test]
async fn test_index_status() {
    let response = reqwest::get("http://0.0.0.0:7002/index/status")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["total_books"].is_number());
    assert!(body["total_words"].is_number());
    assert!(body["last_updated"].is_string());
}

#[tokio::test]
async fn test_index_rebuild() {
    let client = reqwest::Client::new();

    let response = client
        .post("http://0.0.0.0:7002/index/rebuild")
        .send()
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "rebuilt");
    assert!(body["indexed_count"].is_number());
}

#[tokio::test]
async fn test_index_update_non_existing_book() {
    let client = reqwest::Client::new();
    let book_id = "999999";

    let response = client
        .post(&format!("http://0.0.0.0:7002/index/update/{}", book_id))
        .send()
        .await
        .expect("Failed to make request");

    // Should return an error for non-existing book
    assert!(response.status().is_server_error());
}