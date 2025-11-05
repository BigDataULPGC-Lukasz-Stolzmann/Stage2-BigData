//! Integration Tests for Search Service
//!
//! Simplified tests that focus on basic functionality.

use serde_json::Value;

#[tokio::test]
async fn test_health_check() {
    let response = reqwest::get("http://0.0.0.0:7003/status")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "running");
    assert_eq!(body["service"], "search-service");
}

#[tokio::test]
async fn test_basic_search() {
    let response = reqwest::get("http://0.0.0.0:7003/search?q=test")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["query"], "test");
    assert!(body["results"].is_array());
    assert!(body["count"].is_number());
}