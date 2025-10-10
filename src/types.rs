//! Type definitions for the API crawler

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use url::Url;

/// Helper function to check if a HashMap is empty (for serde skip_serializing_if)
fn is_empty_metadata(metadata: &HashMap<String, serde_json::Value>) -> bool {
    metadata.is_empty()
}

/// Helper function to check if a String is empty (for serde skip_serializing_if)
fn is_empty_string(s: &String) -> bool {
    s.is_empty()
}

/// Represents a single API endpoint discovered during crawling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiEndpoint {
    /// The URL of the endpoint
    pub href: String,

    /// The relationship type (e.g., "self", "next", "related")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,

    /// HTTP method if specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// Content type if specified
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// Title or description if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The depth at which this endpoint was discovered
    pub depth: usize,

    /// The parent URL that led to discovering this endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_url: Option<String>,

    /// Additional metadata found in the response
    #[serde(skip_serializing_if = "is_empty_metadata")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ApiEndpoint {
    /// Create a new ApiEndpoint with required fields
    pub fn new(href: String, depth: usize) -> Self {
        Self {
            href,
            rel: None,
            method: None,
            r#type: None,
            title: None,
            depth,
            parent_url: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the relationship type
    pub fn with_rel(mut self, rel: Option<String>) -> Self {
        self.rel = rel;
        self
    }

    /// Set the parent URL
    pub fn with_parent(mut self, parent_url: Option<String>) -> Self {
        self.parent_url = parent_url;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if this endpoint should be crawled (not "self" relation)
    pub fn should_crawl(&self) -> bool {
        self.rel.as_deref() != Some("self")
    }
}

/// Configuration for the API crawler
#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    /// Maximum depth to crawl (0 means unlimited)
    pub max_depth: usize,

    /// Maximum number of concurrent requests
    pub max_concurrent_requests: usize,

    /// Request timeout in seconds
    pub timeout_seconds: u64,

    /// Maximum number of URLs to crawl (0 means unlimited)
    pub max_urls: usize,

    /// User agent string for requests
    pub user_agent: String,

    /// Additional headers to include in requests
    pub headers: HashMap<String, String>,

    /// Delay between requests in milliseconds
    pub delay_ms: u64,

    /// Whether to follow redirects
    pub follow_redirects: bool,

    /// Domains to restrict crawling to (empty means no restriction)
    pub allowed_domains: HashSet<String>,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            max_concurrent_requests: 10,
            timeout_seconds: 30,
            max_urls: 1000,
            user_agent: "API-Crawler/1.0".to_string(),
            headers: HashMap::new(),
            delay_ms: 100,
            follow_redirects: true,
            allowed_domains: HashSet::new(),
        }
    }
}

impl CrawlerConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum crawling depth
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set maximum concurrent requests
    pub fn max_concurrent_requests(mut self, max: usize) -> Self {
        self.max_concurrent_requests = max;
        self
    }

    /// Set request timeout
    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Add an allowed domain
    pub fn allow_domain(mut self, domain: String) -> Self {
        self.allowed_domains.insert(domain);
        self
    }

    /// Add a custom header
    pub fn add_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
}

/// Helper function to check if a Vec is empty (for serde skip_serializing_if)
fn is_empty_errors(errors: &Vec<String>) -> bool {
    errors.is_empty()
}

/// Helper function to check if a usize is zero (for serde skip_serializing_if)
fn is_zero_usize(value: &usize) -> bool {
    *value == 0
}

/// Helper function to check if a u128 is zero (for serde skip_serializing_if)
fn is_zero_u128(value: &u128) -> bool {
    *value == 0
}

/// Statistics about the crawling process
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrawlStats {
    /// Total number of URLs processed
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub urls_processed: usize,

    /// Number of successful requests
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub successful_requests: usize,

    /// Number of failed requests
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub failed_requests: usize,

    /// Number of URLs skipped (duplicate or filtered)
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub urls_skipped: usize,

    /// Maximum depth reached
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub max_depth_reached: usize,

    /// Total time taken for crawling
    #[serde(skip_serializing_if = "is_zero_u128")]
    pub total_time_ms: u128,

    /// Errors encountered during crawling
    #[serde(skip_serializing_if = "is_empty_errors")]
    pub errors: Vec<String>,
}

/// Complete result of the crawling process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResult {
    /// The starting URL that was crawled
    pub start_url: String,

    /// All discovered endpoints
    pub endpoints: Vec<ApiEndpoint>,

    /// Mapping of URLs to their discovered endpoints
    pub url_mappings: HashMap<String, Vec<ApiEndpoint>>,

    /// Statistics about the crawl
    pub stats: CrawlStats,

    /// Timestamp when crawling started
    pub started_at: chrono::DateTime<chrono::Utc>,

    /// Timestamp when crawling completed
    pub completed_at: chrono::DateTime<chrono::Utc>,

    /// Configuration used for this crawl
    #[serde(skip_serializing_if = "is_empty_string")]
    pub config_snapshot: String,
}

impl CrawlResult {
    /// Create a new CrawlResult
    pub fn new(start_url: String, config: &CrawlerConfig) -> Self {
        let now = chrono::Utc::now();
        Self {
            start_url,
            endpoints: Vec::new(),
            url_mappings: HashMap::new(),
            stats: CrawlStats::default(),
            started_at: now,
            completed_at: now,
            config_snapshot: format!("{:?}", config),
        }
    }

    /// Add an endpoint to the result
    pub fn add_endpoint(&mut self, endpoint: ApiEndpoint) {
        // Add to endpoints list
        self.endpoints.push(endpoint.clone());

        // Add to URL mappings
        if let Some(parent) = &endpoint.parent_url {
            self.url_mappings
                .entry(parent.clone())
                .or_insert_with(Vec::new)
                .push(endpoint);
        }
    }

    /// Mark the crawl as completed
    pub fn complete(&mut self) {
        self.completed_at = chrono::Utc::now();
        self.stats.total_time_ms = (self.completed_at - self.started_at).num_milliseconds() as u128;
    }

    /// Get endpoints at a specific depth
    pub fn endpoints_at_depth(&self, depth: usize) -> Vec<&ApiEndpoint> {
        self.endpoints.iter().filter(|e| e.depth == depth).collect()
    }

    /// Get unique domains discovered
    pub fn discovered_domains(&self) -> HashSet<String> {
        self.endpoints
            .iter()
            .filter_map(|e| Url::parse(&e.href).ok())
            .filter_map(|u| u.domain().map(|d| d.to_string()))
            .collect()
    }

    /// Get summary statistics
    pub fn summary(&self) -> String {
        format!(
            "Crawled {} URLs, found {} endpoints across {} domains in {}ms",
            self.stats.urls_processed,
            self.endpoints.len(),
            self.discovered_domains().len(),
            self.stats.total_time_ms
        )
    }
}

/// A queue item for URLs to be processed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueueItem {
    /// The URL to process
    pub url: String,

    /// The depth of this URL
    pub depth: usize,

    /// The parent URL that led to this one
    pub parent_url: Option<String>,
}

impl QueueItem {
    /// Create a new queue item
    pub fn new(url: String, depth: usize, parent_url: Option<String>) -> Self {
        Self {
            url,
            depth,
            parent_url,
        }
    }
}
