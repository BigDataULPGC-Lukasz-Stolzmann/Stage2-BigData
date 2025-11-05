//! Integration Tests for Indexing Service
//!
//! These async tests validate the **Indexing Service** REST API end-to-end,
//! ensuring proper behavior of health checks, book indexing, index rebuilding,
//! and concurrent processing.

use serde_json::Value;
use tokio::time::{sleep, Duration};

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
async fn test_index_update_valid_book() {
    let client = reqwest::Client::new();
    let book_id = "1342";

    let response = client
        .post(&format!("http://0.0.0.0:7002/index/update/{}", book_id))
        .send()
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["book_id"], book_id.parse::<u32>().unwrap());
    assert_eq!(body["status"], "updated");
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

    assert!(response.status().is_client_error() || response.status().is_server_error());
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
async fn test_concurrent_indexing() {
    let client = reqwest::Client::new();
    let book_ids = vec!["84", "11", "1342"];
    let mut handles = vec![];

    for book_id in book_ids {
        let client_clone = client.clone();
        let book_id_clone = book_id.to_string();

        let handle = tokio::spawn(async move {
            let response = client_clone
                .post(&format!("http://0.0.0.0:7002/index/update/{}", book_id_clone))
                .send()
                .await
                .expect("Failed to make request");

            (book_id_clone, response.status())
        });

        handles.push(handle);
    }

    for handle in handles {
        let (book_id, status) = handle.await.expect("Task failed");
        assert_eq!(status, 200, "Book {} failed to index", book_id);
    }
}

#[tokio::test]
async fn test_indexing_workflow() {
    let client = reqwest::Client::new();
    let book_id = "74"; // Adventures of Tom Sawyer

    // First ensure book is ingested (this test assumes ingestion service is running)
    let _ingest_response = client
        .post(&format!("http://0.0.0.0:7001/ingest/{}", book_id))
        .send()
        .await
        .expect("Failed to ingest book");

    // Wait for ingestion to complete
    sleep(Duration::from_secs(2)).await;

    // Now index the book
    let response = client
        .post(&format!("http://0.0.0.0:7002/index/update/{}", book_id))
        .send()
        .await
        .expect("Failed to index book");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["book_id"], book_id.parse::<u32>().unwrap());
    assert_eq!(body["status"], "updated");

    // Verify the index was updated
    let status_response = reqwest::get("http://0.0.0.0:7002/index/status")
        .await
        .expect("Failed to get index status");

    let status_body: Value = status_response.json().await.expect("Failed to parse JSON");
    let total_books = status_body["total_books"].as_u64().unwrap();
    assert!(total_books > 0);
}