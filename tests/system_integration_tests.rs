// ============================================================
//  Simplified system integration tests
//
//  Just tests that all services are up and responding.
// ============================================================

use serde_json::Value;

const INGESTION_BASE_URL: &str = "http://0.0.0.0:7001";
const INDEXING_BASE_URL: &str = "http://0.0.0.0:7002";
const SEARCH_BASE_URL: &str = "http://0.0.0.0:7003";

#[tokio::test]
async fn test_all_services_health() {
    let client = reqwest::Client::new();

    // Test ingestion service
    let response = client.get(&format!("{}/status", INGESTION_BASE_URL)).send().await.expect("Failed to reach ingestion service");
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["service"], "ingestion-service");

    // Test indexing service
    let response = client.get(&format!("{}/status", INDEXING_BASE_URL)).send().await.expect("Failed to reach indexing service");
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["service"], "indexing-service");

    // Test search service
    let response = client.get(&format!("{}/status", SEARCH_BASE_URL)).send().await.expect("Failed to reach search service");
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["service"], "search-service");
}