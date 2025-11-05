//! Integration Tests for Ingestion Service
//!
//! These tests verify that the ingestion service endpoints behave correctly
//! when interacting with live HTTP routes.
//!
//! ## Tested Endpoints
//! - `GET /status` → Health check
//! - `POST /ingest/:book_id` → Book ingestion workflow
//! - `GET /ingest/status/:book_id` → Book status lookup
//! - `GET /ingest/list` → Listing of downloaded books

use serde_json::Value;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_health_check() {
    let response = reqwest::get("http://0.0.0.0:7001/status")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "running");
    assert_eq!(body["service"], "ingestion-service");
}

#[tokio::test]
async fn test_ingest_book_valid_id() {
    let client = reqwest::Client::new();
    let book_id = "1342"; // Pride and Prejudice

    let response = client
        .post(&format!("http://0.0.0.0:7001/ingest/{}", book_id))
        .send()
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["book_id"], book_id.parse::<u32>().unwrap());
    assert_eq!(body["status"], "downloaded");
    assert!(body["path"].is_string()); // Just check path exists
}

#[tokio::test]
async fn test_ingest_book_invalid_id() {
    let client = reqwest::Client::new();
    let book_id = "999999"; // Non-existent book

    let response = client
        .post(&format!("http://0.0.0.0:7001/ingest/{}", book_id))
        .send()
        .await
        .expect("Failed to make request");

    assert!(response.status().is_client_error() || response.status().is_server_error());
}

#[tokio::test]
async fn test_ingest_status_existing_book() {
    let client = reqwest::Client::new();
    let book_id = "1342";

    // First ingest the book
    let _ingest_response = client
        .post(&format!("http://0.0.0.0:7001/ingest/{}", book_id))
        .send()
        .await
        .expect("Failed to ingest book");

    // Wait a moment for processing
    sleep(Duration::from_millis(500)).await;

    // Check status
    let response = client
        .get(&format!("http://0.0.0.0:7001/ingest/status/{}", book_id))
        .send()
        .await
        .expect("Failed to check status");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["book_id"], book_id.parse::<u32>().unwrap());
    // Check if response contains expected fields
    assert!(body.get("book_id").is_some());
}

#[tokio::test]
async fn test_ingest_status_non_existing_book() {
    let book_id = "999998";

    let response = reqwest::get(&format!("http://0.0.0.0:7001/ingest/status/{}", book_id))
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["book_id"], book_id.parse::<u32>().unwrap());
    // Check if response contains expected fields
    assert!(body.get("book_id").is_some());
}

#[tokio::test]
async fn test_list_books() {
    let response = reqwest::get("http://0.0.0.0:7001/ingest/list")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["books"].is_array());
    assert!(body["count"].is_number());
}

#[tokio::test]
async fn test_concurrent_ingestion() {
    let client = reqwest::Client::new();
    let book_ids = vec!["84", "11", "74"];
    let mut handles = vec![];

    for book_id in book_ids {
        let client_clone = client.clone();
        let book_id_clone = book_id.to_string();

        let handle = tokio::spawn(async move {
            let response = client_clone
                .post(&format!("http://localhost:7001/ingest/{}", book_id_clone))
                .send()
                .await
                .expect("Failed to make request");

            (book_id_clone, response.status())
        });

        handles.push(handle);
    }

    for handle in handles {
        let (book_id, status) = handle.await.expect("Task failed");
        assert_eq!(status, 200, "Book {} failed to ingest", book_id);
    }
}