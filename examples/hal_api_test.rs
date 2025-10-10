//! HAL API Test Example
//!
//! This example demonstrates crawling a mock HAL (Hypertext Application Language)
//! API structure using the API crawler. Since we can't easily set up a real HAL API
//! for testing, this example shows how the crawler would work with typical HAL responses.

use api_crawler::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔗 HAL API Crawler Example");
    println!("==========================");
    println!("This example demonstrates how the API crawler works with");
    println!("HAL (Hypertext Application Language) formatted APIs.\n");

    // Create a configuration optimized for API testing
    let config = CrawlerConfig::new()
        .max_depth(3)
        .max_concurrent_requests(5)
        .timeout_seconds(15)
        .add_header("Accept".to_string(), "application/hal+json".to_string())
        .add_header("User-Agent".to_string(), "API-Crawler-Test/1.0".to_string());

    println!("📋 Configuration:");
    println!("  • Max depth: {}", config.max_depth);
    println!(
        "  • Concurrent requests: {}",
        config.max_concurrent_requests
    );
    println!("  • Timeout: {}s", config.timeout_seconds);
    println!("  • Accept: application/hal+json");
    println!();

    // Create the crawler
    let mut crawler = ApiCrawler::new(config)?;

    // Example of what a HAL API response might look like:
    print_example_hal_structure();

    // Try to crawl a JSON API that might have some link structure
    // JSONPlaceholder has some basic REST structure we can demonstrate with
    let test_urls = vec![
        "https://jsonplaceholder.typicode.com/posts",
        "https://api.github.com", // GitHub API uses HAL-like structures
    ];

    for url in test_urls {
        println!("🚀 Testing with: {}", url);
        println!("{}", "=".repeat(50));

        match crawler.crawl(url).await {
            Ok(result) => {
                display_crawl_results(&result);
            }
            Err(e) => {
                println!("❌ Failed to crawl {}: {}", url, e);
            }
        }

        println!("\n{}\n", "-".repeat(50));

        // Reset crawler state for next test
        crawler = ApiCrawler::new(
            CrawlerConfig::new()
                .max_depth(2)
                .max_concurrent_requests(3)
                .timeout_seconds(10),
        )?;
    }

    print_hal_best_practices();

    Ok(())
}

fn print_example_hal_structure() {
    println!("📖 Example HAL API Structure:");
    println!("{}", "─".repeat(30));

    let example_hal = json!({
        "_links": {
            "self": { "href": "/api/users/1" },
            "next": { "href": "/api/users/2" },
            "prev": { "href": "/api/users/0" },
            "posts": { "href": "/api/users/1/posts" },
            "profile": { "href": "/api/users/1/profile" }
        },
        "_embedded": {
            "posts": [
                {
                    "_links": {
                        "self": { "href": "/api/posts/1" },
                        "author": { "href": "/api/users/1" },
                        "comments": { "href": "/api/posts/1/comments" }
                    },
                    "title": "Example Post"
                }
            ]
        },
        "id": 1,
        "name": "John Doe",
        "email": "john@example.com"
    });

    println!("{}", serde_json::to_string_pretty(&example_hal).unwrap());
    println!();

    println!("In this structure, the crawler would discover:");
    println!("  • /api/users/1 (self - would be skipped)");
    println!("  • /api/users/2 (next - would be crawled)");
    println!("  • /api/users/0 (prev - would be crawled)");
    println!("  • /api/users/1/posts (posts - would be crawled)");
    println!("  • /api/users/1/profile (profile - would be crawled)");
    println!("  • /api/posts/1 (embedded self - would be skipped)");
    println!("  • /api/posts/1/comments (comments - would be crawled)");
    println!();
}

fn display_crawl_results(result: &CrawlResult) {
    println!("✅ Crawl completed!");
    println!("📊 Results Summary:");
    println!("  • URLs processed: {}", result.stats.urls_processed);
    println!("  • Endpoints found: {}", result.endpoints.len());
    println!(
        "  • Success rate: {:.1}%",
        if result.stats.urls_processed > 0 {
            (result.stats.successful_requests as f64 / result.stats.urls_processed as f64) * 100.0
        } else {
            0.0
        }
    );
    println!("  • Max depth reached: {}", result.stats.max_depth_reached);
    println!("  • Total time: {}ms", result.stats.total_time_ms);

    if result.endpoints.is_empty() {
        println!("\n⚠️  No endpoints discovered.");
        println!("   This is normal for APIs that don't use HAL or JSON API formats.");
        println!("   The crawler looks for specific link patterns like:");
        println!("   • _links objects (HAL format)");
        println!("   • links objects (JSON API format)");
        println!("   • href attributes in response objects");
        return;
    }

    println!("\n🔗 Discovered Endpoints:");
    for (i, endpoint) in result.endpoints.iter().enumerate().take(10) {
        println!("  {}. {} (depth: {})", i + 1, endpoint.href, endpoint.depth);
        if let Some(ref rel) = endpoint.rel {
            println!("     └─ Relation: {}", rel);
        }
        if let Some(ref parent) = endpoint.parent_url {
            println!("     └─ From: {}", parent);
        }
    }

    if result.endpoints.len() > 10 {
        println!("  ... and {} more endpoints", result.endpoints.len() - 10);
    }

    // Show breakdown by relation type
    let mut rel_counts = std::collections::HashMap::new();
    for endpoint in &result.endpoints {
        let rel = endpoint.rel.as_deref().unwrap_or("(none)");
        *rel_counts.entry(rel).or_insert(0) += 1;
    }

    if !rel_counts.is_empty() {
        println!("\n📋 Endpoints by Relation Type:");
        let mut rels: Vec<_> = rel_counts.iter().collect();
        rels.sort_by(|a, b| b.1.cmp(a.1));

        for (rel, count) in rels {
            println!("  • {}: {}", rel, count);
        }
    }

    // Show any errors
    if !result.stats.errors.is_empty() {
        println!("\n⚠️  Errors encountered:");
        for (i, error) in result.stats.errors.iter().enumerate().take(3) {
            println!("  {}. {}", i + 1, error);
        }
        if result.stats.errors.len() > 3 {
            println!("  ... and {} more errors", result.stats.errors.len() - 3);
        }
    }
}

fn print_hal_best_practices() {
    println!("💡 Best Practices for API Crawling:");
    println!("{}", "=".repeat(35));

    println!("\n🎯 For HAL APIs:");
    println!("  • Use 'Accept: application/hal+json' header");
    println!("  • Look for _links and _embedded objects");
    println!("  • Relations like 'next', 'prev' indicate pagination");
    println!("  • 'self' relations are automatically skipped");

    println!("\n🎯 For JSON API:");
    println!("  • Use 'Accept: application/vnd.api+json' header");
    println!("  • Look for 'links' objects in responses");
    println!("  • Support for relationship links");

    println!("\n🎯 General Tips:");
    println!("  • Start with low concurrency (2-5) for unknown APIs");
    println!("  • Use authentication headers for protected endpoints");
    println!("  • Set reasonable timeouts (10-30 seconds)");
    println!("  • Monitor for rate limiting responses");
    println!("  • Use domain restrictions to stay within API boundaries");

    println!("\n🔧 Example CLI Commands:");
    println!("  # Basic HAL API crawl");
    println!("  ./api_crawler https://api.example.com --header 'Accept:application/hal+json'");

    println!("\n  # Authenticated crawl with rate limiting");
    println!("  ./api_crawler https://api.example.com \\");
    println!("    --header 'Authorization:Bearer TOKEN' \\");
    println!("    --header 'Accept:application/hal+json' \\");
    println!("    --concurrency 2 --delay 500");

    println!("\n  # Conservative deep crawl");
    println!("  ./api_crawler https://api.example.com \\");
    println!("    --max-depth 5 --max-urls 100 \\");
    println!("    --timeout 60 --output results.json");
}

#[cfg(test)]
mod tests {
    use super::*;
    use api_crawler::types::ApiEndpoint;

    #[test]
    fn test_hal_link_pattern() {
        // Test that we can identify HAL-style links
        let endpoint =
            ApiEndpoint::new("/api/users/1".to_string(), 1).with_rel(Some("self".to_string()));

        // Self relations should not be crawled
        assert!(!endpoint.should_crawl());

        let endpoint2 =
            ApiEndpoint::new("/api/users/2".to_string(), 1).with_rel(Some("next".to_string()));

        // Next relations should be crawled
        assert!(endpoint2.should_crawl());
    }

    #[test]
    fn test_crawler_config_for_hal() {
        let config = CrawlerConfig::new()
            .max_depth(3)
            .max_concurrent_requests(5)
            .add_header("Accept".to_string(), "application/hal+json".to_string());

        assert_eq!(config.max_depth, 3);
        assert_eq!(config.max_concurrent_requests, 5);
        assert!(config.headers.contains_key("Accept"));
        assert_eq!(
            config.headers.get("Accept").unwrap(),
            "application/hal+json"
        );
    }

    #[tokio::test]
    async fn test_crawler_creation_with_hal_config() {
        let config = CrawlerConfig::new()
            .add_header("Accept".to_string(), "application/hal+json".to_string())
            .add_header("User-Agent".to_string(), "Test-Crawler/1.0".to_string());

        let crawler = ApiCrawler::new(config);
        assert!(crawler.is_ok());
    }
}
