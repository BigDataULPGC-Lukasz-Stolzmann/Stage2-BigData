// ============================================================
//  End-to-end tests for the Big Data Search Engine system.
//
//  Covers:
//    - Full book ingestion → indexing → search workflow
//    - Multi-book and concurrent operations
//    - Service resilience and error handling
//    - End-to-end performance metrics
//
//  Requires all services running locally or via Docker:
//    ingestion-service (port 7001)
//    indexing-service  (port 7002)
//    search-service    (port 7003)
// ============================================================

use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

const INGESTION_BASE_URL: &str = "http://0.0.0.0:7001";
const INDEXING_BASE_URL: &str = "http://0.0.0.0:7002";
const SEARCH_BASE_URL: &str = "http://0.0.0.0:7003";

async fn wait_for_services() {
    let client = reqwest::Client::new();
    let services = [
        (INGESTION_BASE_URL, "ingestion"),
        (INDEXING_BASE_URL, "indexing"),
        (SEARCH_BASE_URL, "search"),
    ];

    for (url, name) in &services {
        let mut attempts = 0;
        loop {
            match client.get(&format!("{}/status", url)).send().await {
                Ok(response) if response.status().is_success() => {
                    println!("{} service is ready", name);
                    break;
                }
                _ => {
                    attempts += 1;
                    if attempts > 30 {
                        panic!("{} service failed to start after 30 attempts", name);
                    }
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}

#[tokio::test]
async fn test_complete_book_processing_workflow() {
    wait_for_services().await;

    let client = reqwest::Client::new();
    let book_id = "1342"; // Pride and Prejudice

    // Step 1: Ingest the book
    println!("Step 1: Ingesting book {}", book_id);
    let ingest_response = client
        .post(&format!("{}/ingest/{}", INGESTION_BASE_URL, book_id))
        .send()
        .await
        .expect("Failed to ingest book");

    assert_eq!(ingest_response.status(), 200);
    let ingest_body: Value = ingest_response
        .json()
        .await
        .expect("Failed to parse ingest response");
    assert_eq!(ingest_body["book_id"], book_id);
    assert_eq!(ingest_body["status"], "downloaded");

    // Wait for ingestion to complete
    sleep(Duration::from_secs(2)).await;

    // Step 2: Verify book is available
    println!("Step 2: Verifying book availability");
    let status_response = client
        .get(&format!("{}/ingest/status/{}", INGESTION_BASE_URL, book_id))
        .send()
        .await
        .expect("Failed to check book status");

    assert_eq!(status_response.status(), 200);
    let status_body: Value = status_response
        .json()
        .await
        .expect("Failed to parse status response");
    assert_eq!(status_body["available"], true);

    // Step 3: Index the book
    println!("Step 3: Indexing book {}", book_id);
    let index_response = client
        .post(&format!("{}/index/update/{}", INDEXING_BASE_URL, book_id))
        .send()
        .await
        .expect("Failed to index book");

    assert_eq!(index_response.status(), 200);
    let index_body: Value = index_response
        .json()
        .await
        .expect("Failed to parse index response");
    assert_eq!(index_body["book_id"], book_id);
    assert_eq!(index_body["status"], "updated");

    // Wait for indexing to complete
    sleep(Duration::from_secs(3)).await;

    // Step 4: Search for the book
    println!("Step 4: Searching for book content");
    let search_response = client
        .get(&format!("{}/search?q=pride", SEARCH_BASE_URL))
        .send()
        .await
        .expect("Failed to search");

    assert_eq!(search_response.status(), 200);
    let search_body: Value = search_response
        .json()
        .await
        .expect("Failed to parse search response");

    assert_eq!(search_body["query"], "pride");
    assert!(search_body["count"].as_u64().unwrap() > 0);

    let results = search_body["results"].as_array().unwrap();
    assert!(!results.is_empty());

    // Verify that our book is in the results
    let book_found = results
        .iter()
        .any(|result| result["book_id"].as_str().unwrap() == book_id);
    assert!(
        book_found,
        "Book {} should be found in search results",
        book_id
    );

    println!("✅ Complete workflow test passed!");
}

#[tokio::test]
async fn test_multiple_books_workflow() {
    wait_for_services().await;

    let client = reqwest::Client::new();
    let book_ids = vec!["84", "11", "74"]; // Frankenstein, Alice in Wonderland, Tom Sawyer

    // Step 1: Ingest all books
    println!("Step 1: Ingesting multiple books");
    for book_id in &book_ids {
        let response = client
            .post(&format!("{}/ingest/{}", INGESTION_BASE_URL, book_id))
            .send()
            .await
            .expect("Failed to ingest book");

        assert_eq!(response.status(), 200);
        println!("Ingested book {}", book_id);
    }

    // Wait for all ingestions to complete
    sleep(Duration::from_secs(5)).await;

    // Step 2: Index all books
    println!("Step 2: Indexing all books");
    for book_id in &book_ids {
        let response = client
            .post(&format!("{}/index/update/{}", INDEXING_BASE_URL, book_id))
            .send()
            .await
            .expect("Failed to index book");

        assert_eq!(response.status(), 200);
        println!("Indexed book {}", book_id);
    }

    // Wait for all indexing to complete
    sleep(Duration::from_secs(10)).await;

    // Step 3: Verify all books are searchable
    println!("Step 3: Verifying all books are searchable");
    let search_terms = vec!["alice", "frankenstein", "adventure"];

    for term in &search_terms {
        let response = client
            .get(&format!("{}/search?q={}", SEARCH_BASE_URL, term))
            .send()
            .await
            .expect("Failed to search");

        assert_eq!(response.status(), 200);
        let body: Value = response
            .json()
            .await
            .expect("Failed to parse search response");

        // Should have some results for common terms
        if body["count"].as_u64().unwrap() > 0 {
            println!("✅ Found results for '{}'", term);
        }
    }

    // Step 4: Test search with filters
    println!("Step 4: Testing search with filters");
    let response = client
        .get(&format!(
            "{}/search?q=adventure&language=en",
            SEARCH_BASE_URL
        ))
        .send()
        .await
        .expect("Failed to search with filters");

    assert_eq!(response.status(), 200);
    let body: Value = response
        .json()
        .await
        .expect("Failed to parse filtered search response");
    assert_eq!(body["filters"]["language"], "en");

    println!("✅ Multiple books workflow test passed!");
}

#[tokio::test]
async fn test_system_resilience() {
    wait_for_services().await;

    let client = reqwest::Client::new();

    // Test 1: Try to index a non-existent book
    println!("Test 1: Indexing non-existent book");
    let response = client
        .post(&format!("{}/index/update/999999", INDEXING_BASE_URL))
        .send()
        .await
        .expect("Failed to make request");

    assert!(response.status().is_client_error() || response.status().is_server_error());

    // Test 2: Search with empty query
    println!("Test 2: Search with empty query");
    let response = client
        .get(&format!("{}/search?q=", SEARCH_BASE_URL))
        .send()
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 400);

    // Test 3: Get index status (should always work)
    println!("Test 3: Getting index status");
    let response = client
        .get(&format!("{}/index/status", INDEXING_BASE_URL))
        .send()
        .await
        .expect("Failed to get index status");

    assert_eq!(response.status(), 200);
    let body: Value = response
        .json()
        .await
        .expect("Failed to parse status response");
    assert!(body["total_books"].is_number());

    // Test 4: List books (should always work)
    println!("Test 4: Listing books");
    let response = client
        .get(&format!("{}/ingest/list", INGESTION_BASE_URL))
        .send()
        .await
        .expect("Failed to list books");

    assert_eq!(response.status(), 200);
    let body: Value = response
        .json()
        .await
        .expect("Failed to parse list response");
    assert!(body["books"].is_array());

    println!("✅ System resilience test passed!");
}

#[tokio::test]
async fn test_concurrent_operations() {
    wait_for_services().await;

    let client = reqwest::Client::new();
    let book_ids = vec!["1080", "2701", "345"]; // Multiple books for concurrent testing

    // Test concurrent ingestion
    println!("Testing concurrent ingestion");
    let ingest_handles: Vec<_> = book_ids
        .iter()
        .map(|&book_id| {
            let client = client.clone();
            tokio::spawn(async move {
                let response = client
                    .post(&format!("{}/ingest/{}", INGESTION_BASE_URL, book_id))
                    .send()
                    .await
                    .expect("Failed to ingest book");
                (book_id, response.status())
            })
        })
        .collect();

    for handle in ingest_handles {
        let (book_id, status) = handle.await.expect("Ingest task failed");
        assert_eq!(
            status, 200,
            "Concurrent ingestion failed for book {}",
            book_id
        );
    }

    // Wait for all ingestions
    sleep(Duration::from_secs(5)).await;

    // Test concurrent indexing
    println!("Testing concurrent indexing");
    let index_handles: Vec<_> = book_ids
        .iter()
        .map(|&book_id| {
            let client = client.clone();
            tokio::spawn(async move {
                let response = client
                    .post(&format!("{}/index/update/{}", INDEXING_BASE_URL, book_id))
                    .send()
                    .await
                    .expect("Failed to index book");
                (book_id, response.status())
            })
        })
        .collect();

    for handle in index_handles {
        let (book_id, status) = handle.await.expect("Index task failed");
        assert_eq!(
            status, 200,
            "Concurrent indexing failed for book {}",
            book_id
        );
    }

    // Wait for all indexing
    sleep(Duration::from_secs(10)).await;

    // Test concurrent searches
    println!("Testing concurrent searches");
    let search_terms = vec!["love", "death", "life", "hope", "fear"];
    let search_handles: Vec<_> = search_terms
        .iter()
        .map(|&term| {
            let client = client.clone();
            tokio::spawn(async move {
                let response = client
                    .get(&format!("{}/search?q={}", SEARCH_BASE_URL, term))
                    .send()
                    .await
                    .expect("Failed to search");
                (term, response.status())
            })
        })
        .collect();

    for handle in search_handles {
        let (term, status) = handle.await.expect("Search task failed");
        assert_eq!(status, 200, "Concurrent search failed for term {}", term);
    }

    println!("✅ Concurrent operations test passed!");
}

#[tokio::test]
async fn test_end_to_end_performance() {
    wait_for_services().await;

    let client = reqwest::Client::new();
    let book_id = "2600"; // War and Peace (large book for performance testing)

    let start_time = std::time::Instant::now();

    // Ingest
    let ingest_start = std::time::Instant::now();
    let response = client
        .post(&format!("{}/ingest/{}", INGESTION_BASE_URL, book_id))
        .send()
        .await
        .expect("Failed to ingest book");
    assert_eq!(response.status(), 200);
    let ingest_duration = ingest_start.elapsed();

    sleep(Duration::from_secs(3)).await;

    // Index
    let index_start = std::time::Instant::now();
    let response = client
        .post(&format!("{}/index/update/{}", INDEXING_BASE_URL, book_id))
        .send()
        .await
        .expect("Failed to index book");
    assert_eq!(response.status(), 200);
    let index_duration = index_start.elapsed();

    sleep(Duration::from_secs(5)).await;

    // Search
    let search_start = std::time::Instant::now();
    let response = client
        .get(&format!("{}/search?q=war", SEARCH_BASE_URL))
        .send()
        .await
        .expect("Failed to search");
    assert_eq!(response.status(), 200);
    let search_duration = search_start.elapsed();

    let total_duration = start_time.elapsed();

    println!("📊 Performance Results:");
    println!("  Ingestion: {:?}", ingest_duration);
    println!("  Indexing: {:?}", index_duration);
    println!("  Search: {:?}", search_duration);
    println!("  Total: {:?}", total_duration);

    // Basic performance assertions (adjust based on your requirements)
    assert!(
        ingest_duration < Duration::from_secs(30),
        "Ingestion took too long"
    );
    assert!(
        index_duration < Duration::from_secs(60),
        "Indexing took too long"
    );
    assert!(
        search_duration < Duration::from_secs(5),
        "Search took too long"
    );

    println!("✅ End-to-end performance test passed!");
}
