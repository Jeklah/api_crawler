//! Core API crawler implementation

use crate::error::{CrawlerError, Result};
use crate::types::{ApiEndpoint, CrawlResult, CrawlerConfig, QueueItem};
use reqwest::Client;
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::{Instant, sleep};
use tracing::{debug, error, info};
use url::Url;

/// The main API crawler
pub struct ApiCrawler {
    /// HTTP client for making requests
    client: Client,

    /// Configuration for the crawler
    config: CrawlerConfig,

    /// Semaphore to limit concurrent requests
    semaphore: Arc<Semaphore>,

    /// Set of URLs we've already visited to prevent loops
    visited_urls: HashSet<String>,

    /// Queue of URLs to process
    url_queue: VecDeque<QueueItem>,
}

impl ApiCrawler {
    /// Create a new API crawler with the given configuration
    pub fn new(config: CrawlerConfig) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();

        // Add user agent
        headers.insert(
            reqwest::header::USER_AGENT,
            config
                .user_agent
                .parse()
                .map_err(|_| CrawlerError::config("Invalid user agent"))?,
        );

        // Add custom headers
        for (key, value) in &config.headers {
            let header_name: reqwest::header::HeaderName = key
                .parse()
                .map_err(|_| CrawlerError::config(format!("Invalid header name: {}", key)))?;
            let header_value: reqwest::header::HeaderValue = value
                .parse()
                .map_err(|_| CrawlerError::config(format!("Invalid header value: {}", value)))?;
            headers.insert(header_name, header_value);
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .default_headers(headers)
            .redirect(if config.follow_redirects {
                reqwest::redirect::Policy::limited(10)
            } else {
                reqwest::redirect::Policy::none()
            })
            .build()?;

        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));

        Ok(Self {
            client,
            semaphore,
            visited_urls: HashSet::new(),
            url_queue: VecDeque::new(),
            config,
        })
    }

    /// Start crawling from the given URL
    pub async fn crawl(&mut self, start_url: &str) -> Result<CrawlResult> {
        info!("Starting crawl from: {}", start_url);

        let mut result = CrawlResult::new(start_url.to_string(), &self.config);
        let start_time = Instant::now();

        // Validate and normalize the starting URL
        let start_url = self.normalize_url(start_url)?;

        // Add the starting URL to the queue
        self.url_queue.push_back(QueueItem::new(start_url, 0, None));

        while let Some(item) = self.url_queue.pop_front() {
            // Check limits
            if self.config.max_depth > 0 && item.depth >= self.config.max_depth {
                debug!(
                    "Reached maximum depth {} for URL: {}",
                    self.config.max_depth, item.url
                );
                result.stats.urls_skipped += 1;
                continue;
            }

            if self.config.max_urls > 0 && result.stats.urls_processed >= self.config.max_urls {
                debug!("Reached maximum URL limit: {}", self.config.max_urls);
                break;
            }

            // Skip if already visited
            if self.visited_urls.contains(&item.url) {
                debug!("Skipping already visited URL: {}", item.url);
                result.stats.urls_skipped += 1;
                continue;
            }

            // Check domain restrictions
            if !self.is_domain_allowed(&item.url)? {
                debug!("Skipping URL due to domain restriction: {}", item.url);
                result.stats.urls_skipped += 1;
                continue;
            }

            // Mark as visited
            self.visited_urls.insert(item.url.clone());

            // Process the URL
            match self.process_url(&item).await {
                Ok(endpoints) => {
                    result.stats.successful_requests += 1;
                    result.stats.urls_processed += 1;
                    result.stats.max_depth_reached = result.stats.max_depth_reached.max(item.depth);

                    info!("Found {} endpoints at {}", endpoints.len(), item.url);

                    for endpoint in endpoints {
                        // Add to results
                        result.add_endpoint(endpoint.clone());

                        // Queue for further crawling if it should be crawled
                        if endpoint.should_crawl() {
                            let queue_item = QueueItem::new(
                                endpoint.href.clone(),
                                item.depth + 1,
                                Some(item.url.clone()),
                            );

                            if !self.visited_urls.contains(&endpoint.href) {
                                self.url_queue.push_back(queue_item);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to process URL {}: {}", item.url, e);
                    result.stats.failed_requests += 1;
                    result.stats.errors.push(format!("URL {}: {}", item.url, e));
                }
            }

            // Add delay between requests
            if self.config.delay_ms > 0 {
                sleep(Duration::from_millis(self.config.delay_ms)).await;
            }
        }

        result.complete();

        info!(
            "Crawling completed. Processed {} URLs, found {} endpoints in {}ms",
            result.stats.urls_processed,
            result.endpoints.len(),
            start_time.elapsed().as_millis()
        );

        Ok(result)
    }

    /// Process a single URL and extract endpoints
    async fn process_url(&self, item: &QueueItem) -> Result<Vec<ApiEndpoint>> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| CrawlerError::config("Failed to acquire semaphore permit"))?;

        debug!("Processing URL at depth {}: {}", item.depth, item.url);

        // Make HTTP request
        let response = self.client.get(&item.url).send().await?;

        // Check if response is JSON
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("application/json")
            && !content_type.contains("application/hal+json")
        {
            debug!("Skipping non-JSON response from {}", item.url);
            return Ok(Vec::new());
        }

        // Parse JSON response
        let json: Value = response.json().await?;

        // Extract endpoints from JSON
        self.extract_endpoints_from_json(&json, item)
    }

    /// Extract API endpoints from a JSON response
    fn extract_endpoints_from_json(
        &self,
        json: &Value,
        parent_item: &QueueItem,
    ) -> Result<Vec<ApiEndpoint>> {
        let mut endpoints = Vec::new();

        match json {
            Value::Object(obj) => {
                // Look for common patterns in REST APIs
                self.extract_from_object(obj, parent_item, &mut endpoints)?;
            }
            Value::Array(arr) => {
                // Process each item in the array
                for item in arr {
                    if let Value::Object(obj) = item {
                        self.extract_from_object(obj, parent_item, &mut endpoints)?;
                    }
                }
            }
            _ => {
                debug!("JSON response is not an object or array, skipping endpoint extraction");
            }
        }

        Ok(endpoints)
    }

    /// Extract endpoints from a JSON object
    fn extract_from_object(
        &self,
        obj: &serde_json::Map<String, Value>,
        parent_item: &QueueItem,
        endpoints: &mut Vec<ApiEndpoint>,
    ) -> Result<()> {
        // Look for HAL (Hypertext Application Language) style links
        if let Some(Value::Object(links)) = obj.get("_links") {
            for (rel, link_data) in links {
                self.extract_from_link_data(rel, link_data, parent_item, endpoints)?;
            }
        }

        // Look for JSON API style links
        if let Some(links_value) = obj.get("links") {
            match links_value {
                Value::Object(links) => {
                    for (rel, link_data) in links {
                        self.extract_from_link_data(rel, link_data, parent_item, endpoints)?;
                    }
                }
                Value::Array(links_array) => {
                    for link_item in links_array {
                        if let Value::Object(link_obj) = link_item {
                            let rel = link_obj
                                .get("rel")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            self.extract_from_link_data(rel, link_item, parent_item, endpoints)?;
                        }
                    }
                }
                _ => {}
            }
        }

        // Look for direct href properties
        if let Some(Value::String(href)) = obj.get("href") {
            let rel = obj
                .get("rel")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let mut endpoint = ApiEndpoint::new(href.clone(), parent_item.depth + 1)
                .with_rel(rel)
                .with_parent(Some(parent_item.url.clone()));

            // Set known ApiEndpoint fields if they exist in the object
            if let Some(Value::String(method)) = obj.get("method") {
                endpoint.method = Some(method.clone());
            }
            if let Some(Value::String(content_type)) = obj.get("type") {
                endpoint.r#type = Some(content_type.clone());
            }
            if let Some(Value::String(title)) = obj.get("title") {
                endpoint.title = Some(title.clone());
            }

            // Extract additional metadata, excluding known ApiEndpoint fields
            for (key, value) in obj {
                if !matches!(key.as_str(), "href" | "rel" | "method" | "type" | "title") {
                    endpoint = endpoint.with_metadata(key.clone(), value.clone());
                }
            }

            endpoints.push(endpoint);
        }

        // Look for URL patterns in other fields
        for (key, value) in obj {
            if key.contains("url") || key.contains("uri") || key.ends_with("_link") {
                if let Some(url_str) = value.as_str() {
                    if self.looks_like_url(url_str) {
                        let endpoint = ApiEndpoint::new(url_str.to_string(), parent_item.depth + 1)
                            .with_parent(Some(parent_item.url.clone()))
                            .with_metadata(format!("source_field"), Value::String(key.clone()));

                        endpoints.push(endpoint);
                    }
                }
            }
        }

        // Recursively process nested objects and arrays
        for value in obj.values() {
            match value {
                Value::Object(nested_obj) => {
                    self.extract_from_object(nested_obj, parent_item, endpoints)?;
                }
                Value::Array(arr) => {
                    for item in arr {
                        if let Value::Object(nested_obj) = item {
                            self.extract_from_object(nested_obj, parent_item, endpoints)?;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Extract endpoint from link data (could be string or object)
    fn extract_from_link_data(
        &self,
        rel: &str,
        link_data: &Value,
        parent_item: &QueueItem,
        endpoints: &mut Vec<ApiEndpoint>,
    ) -> Result<()> {
        match link_data {
            Value::String(href) => {
                let endpoint = ApiEndpoint::new(href.clone(), parent_item.depth + 1)
                    .with_rel(Some(rel.to_string()))
                    .with_parent(Some(parent_item.url.clone()));
                endpoints.push(endpoint);
            }
            Value::Object(link_obj) => {
                if let Some(Value::String(href)) = link_obj.get("href") {
                    let mut endpoint = ApiEndpoint::new(href.clone(), parent_item.depth + 1)
                        .with_rel(Some(rel.to_string()))
                        .with_parent(Some(parent_item.url.clone()));

                    // Set known ApiEndpoint fields if they exist in the link object
                    if let Some(Value::String(method)) = link_obj.get("method") {
                        endpoint.method = Some(method.clone());
                    }
                    if let Some(Value::String(content_type)) = link_obj.get("type") {
                        endpoint.r#type = Some(content_type.clone());
                    }
                    if let Some(Value::String(title)) = link_obj.get("title") {
                        endpoint.title = Some(title.clone());
                    }

                    // Extract additional link metadata, excluding known ApiEndpoint fields
                    for (key, value) in link_obj {
                        if !matches!(key.as_str(), "href" | "rel" | "method" | "type" | "title") {
                            endpoint = endpoint.with_metadata(key.clone(), value.clone());
                        }
                    }

                    endpoints.push(endpoint);
                }
            }
            Value::Array(link_array) => {
                for link_item in link_array {
                    self.extract_from_link_data(rel, link_item, parent_item, endpoints)?;
                }
            }
            _ => {
                debug!(
                    "Unexpected link data type for rel '{}': {:?}",
                    rel, link_data
                );
            }
        }

        Ok(())
    }

    /// Check if a string looks like a URL
    fn looks_like_url(&self, s: &str) -> bool {
        s.starts_with("http://") || s.starts_with("https://") || s.starts_with("/")
    }

    /// Normalize a URL (convert relative to absolute, etc.)
    fn normalize_url(&self, url: &str) -> Result<String> {
        let parsed = Url::parse(url)?;
        Ok(parsed.to_string())
    }

    /// Check if a domain is allowed based on configuration
    fn is_domain_allowed(&self, url: &str) -> Result<bool> {
        if self.config.allowed_domains.is_empty() {
            return Ok(true);
        }

        let parsed = Url::parse(url)?;
        if let Some(domain) = parsed.domain() {
            Ok(self.config.allowed_domains.contains(domain))
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_endpoint_should_crawl() {
        let endpoint_self = ApiEndpoint::new("http://example.com".to_string(), 1)
            .with_rel(Some("self".to_string()));
        assert!(!endpoint_self.should_crawl());

        let endpoint_next = ApiEndpoint::new("http://example.com/next".to_string(), 1)
            .with_rel(Some("next".to_string()));
        assert!(endpoint_next.should_crawl());

        let endpoint_no_rel = ApiEndpoint::new("http://example.com/other".to_string(), 1);
        assert!(endpoint_no_rel.should_crawl());
    }

    #[test]
    fn test_looks_like_url() {
        let crawler = ApiCrawler::new(CrawlerConfig::default()).unwrap();

        assert!(crawler.looks_like_url("http://example.com"));
        assert!(crawler.looks_like_url("https://example.com"));
        assert!(crawler.looks_like_url("/api/endpoint"));
        assert!(!crawler.looks_like_url("not-a-url"));
        assert!(!crawler.looks_like_url("example.com"));
    }

    #[tokio::test]
    async fn test_extract_endpoints_from_hal_json() {
        let crawler = ApiCrawler::new(CrawlerConfig::default()).unwrap();
        let parent_item = QueueItem::new("http://example.com".to_string(), 0, None);

        let json = json!({
            "_links": {
                "self": {"href": "http://example.com/current"},
                "next": {"href": "http://example.com/next"},
                "items": [
                    {"href": "http://example.com/item1"},
                    {"href": "http://example.com/item2"}
                ]
            }
        });

        let endpoints = crawler
            .extract_endpoints_from_json(&json, &parent_item)
            .unwrap();

        // The extraction process recursively finds endpoints, so we might get more than expected
        // due to the nested structure and multiple extraction patterns
        assert!(endpoints.len() >= 4);

        let self_endpoint = endpoints
            .iter()
            .find(|e| e.rel == Some("self".to_string()))
            .unwrap();
        assert!(!self_endpoint.should_crawl());

        let next_endpoint = endpoints
            .iter()
            .find(|e| e.rel == Some("next".to_string()))
            .unwrap();
        assert!(next_endpoint.should_crawl());
    }

    #[test]
    fn test_no_metadata_duplication() {
        let crawler = ApiCrawler::new(CrawlerConfig::default()).unwrap();
        let parent_item = QueueItem::new("http://example.com".to_string(), 0, None);
        let mut endpoints = Vec::new();

        // Test link object with rel, method, type, title, and custom metadata
        let link_obj = json!({
            "href": "http://example.com/test",
            "rel": "test-rel",
            "method": "POST",
            "type": "application/json",
            "title": "Test Endpoint",
            "custom_field": "custom_value",
            "another_custom": 42
        });

        crawler
            .extract_from_link_data("test-rel", &link_obj, &parent_item, &mut endpoints)
            .unwrap();

        assert_eq!(endpoints.len(), 1);
        let endpoint = &endpoints[0];

        // Verify that known fields are set directly on the endpoint
        assert_eq!(endpoint.rel, Some("test-rel".to_string()));
        assert_eq!(endpoint.method, Some("POST".to_string()));
        assert_eq!(endpoint.r#type, Some("application/json".to_string()));
        assert_eq!(endpoint.title, Some("Test Endpoint".to_string()));

        // Verify that known fields are NOT duplicated in metadata
        assert!(!endpoint.metadata.contains_key("rel"));
        assert!(!endpoint.metadata.contains_key("method"));
        assert!(!endpoint.metadata.contains_key("type"));
        assert!(!endpoint.metadata.contains_key("title"));
        assert!(!endpoint.metadata.contains_key("href"));

        // Verify that custom fields ARE in metadata
        assert_eq!(
            endpoint.metadata.get("custom_field"),
            Some(&json!("custom_value"))
        );
        assert_eq!(endpoint.metadata.get("another_custom"), Some(&json!(42)));
    }
}
