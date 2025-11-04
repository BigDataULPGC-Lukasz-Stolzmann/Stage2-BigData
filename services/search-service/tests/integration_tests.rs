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
    let response = reqwest::get("http://0.0.0.0:7003/search?q=adventure")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["query"], "adventure");
    assert!(body["results"].is_array());
    assert!(body["count"].is_number());
    assert!(body["filters"].is_object());
}

#[tokio::test]
async fn test_search_with_author_filter() {
    let response = reqwest::get("http://0.0.0.0:7003/search?q=pride&author=Jane%20Austen")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["query"], "pride");
    assert_eq!(body["filters"]["author"], "Jane Austen");
    assert!(body["results"].is_array());
}

#[tokio::test]
async fn test_search_with_language_filter() {
    let response = reqwest::get("http://0.0.0.0:7003/search?q=adventure&language=en")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["query"], "adventure");
    assert_eq!(body["filters"]["language"], "en");
    assert!(body["results"].is_array());
}

#[tokio::test]
async fn test_search_with_year_filter() {
    let response = reqwest::get("http://0.0.0.0:7003/search?q=adventure&year=1865")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["query"], "adventure");
    assert_eq!(body["filters"]["year"], 1865);
    assert!(body["results"].is_array());
}

#[tokio::test]
async fn test_search_with_combined_filters() {
    let response = reqwest::get("http://0.0.0.0:7003/search?q=adventure&author=Mark%20Twain&language=en&year=1876")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["query"], "adventure");
    assert_eq!(body["filters"]["author"], "Mark Twain");
    assert_eq!(body["filters"]["language"], "en");
    assert_eq!(body["filters"]["year"], 1876);
    assert!(body["results"].is_array());
}

#[tokio::test]
async fn test_empty_search_query() {
    let response = reqwest::get("http://0.0.0.0:7003/search?q=")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_search_no_results() {
    let response = reqwest::get("http://0.0.0.0:7003/search?q=xyzneverexistingword")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["query"], "xyzneverexistingword");
    assert_eq!(body["count"], 0);
    assert_eq!(body["results"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_search_result_structure() {
    let response = reqwest::get("http://0.0.0.0:7003/search?q=pride")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    let results = body["results"].as_array().unwrap();

    if !results.is_empty() {
        let first_result = &results[0];

        // Verify result structure
        assert!(first_result["book_id"].is_string());
        assert!(first_result["title"].is_string());
        assert!(first_result["author"].is_string());
        assert!(first_result["language"].is_string());
        assert!(first_result["year"].is_number() || first_result["year"].is_null());
        assert!(first_result["score"].is_number());
        assert!(first_result["matches"].is_array());
    }
}

#[tokio::test]
async fn test_search_pagination() {
    // Test with limit parameter if implemented
    let response = reqwest::get("http://0.0.0.0:7003/search?q=the&limit=5")
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    let results = body["results"].as_array().unwrap();

    // If pagination is implemented, results should be limited
    assert!(results.len() <= 5);
}

#[tokio::test]
async fn test_concurrent_searches() {
    let queries = vec!["adventure", "love", "mystery", "science", "history"];
    let mut handles = vec![];

    for query in queries {
        let query_clone = query.to_string();

        let handle = tokio::spawn(async move {
            let response = reqwest::get(&format!("http://0.0.0.0:7003/search?q={}", query_clone))
                .await
                .expect("Failed to make request");

            (query_clone, response.status())
        });

        handles.push(handle);
    }

    for handle in handles {
        let (query, status) = handle.await.expect("Task failed");
        assert_eq!(status, 200, "Search for '{}' failed", query);
    }
}