//! Simple example demonstrating the API crawler library usage
//!
//! This example shows how to use the API crawler programmatically
//! to crawl a mock API and display the results.

use api_crawler::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🕷️  API Crawler Example");
    println!("====================");

    // Create a basic configuration
    let config = CrawlerConfig::new()
        .max_depth(3)
        .max_concurrent_requests(5)
        .timeout_seconds(10);

    println!("Configuration:");
    println!("  Max depth: {}", config.max_depth);
    println!(
        "  Max concurrent requests: {}",
        config.max_concurrent_requests
    );
    println!("  Timeout: {}s", config.timeout_seconds);
    println!();

    // Create the crawler
    let mut crawler = ApiCrawler::new(config)?;

    // Example: Crawl a public API (JSONPlaceholder)
    // This is a free testing API that returns JSON responses
    let start_url = "https://jsonplaceholder.typicode.com/posts/1";

    println!("Starting crawl from: {}", start_url);
    println!("Note: This example uses JSONPlaceholder, a free testing API");
    println!();

    match crawler.crawl(start_url).await {
        Ok(result) => {
            println!("✅ Crawling completed successfully!");
            println!();

            // Display summary
            println!("📊 Summary:");
            println!("  URLs processed: {}", result.stats.urls_processed);
            println!("  Endpoints found: {}", result.endpoints.len());
            println!(
                "  Success rate: {:.1}%",
                if result.stats.urls_processed > 0 {
                    (result.stats.successful_requests as f64 / result.stats.urls_processed as f64)
                        * 100.0
                } else {
                    0.0
                }
            );
            println!("  Total time: {}ms", result.stats.total_time_ms);
            println!();

            // Display discovered endpoints
            if !result.endpoints.is_empty() {
                println!("🔗 Discovered Endpoints:");
                for (i, endpoint) in result.endpoints.iter().enumerate().take(10) {
                    println!("  {}. {}", i + 1, endpoint.href);
                    if let Some(ref rel) = endpoint.rel {
                        println!("     Relation: {}", rel);
                    }
                    println!("     Depth: {}", endpoint.depth);
                    if let Some(ref parent) = endpoint.parent_url {
                        println!("     Parent: {}", parent);
                    }
                    println!();
                }

                if result.endpoints.len() > 10 {
                    println!("  ... and {} more endpoints", result.endpoints.len() - 10);
                }
            } else {
                println!("ℹ️  No additional endpoints discovered.");
                println!("   This might be because:");
                println!("   - The API doesn't use standard link formats (HAL, JSON API, etc.)");
                println!("   - The response doesn't contain href attributes");
                println!("   - The API structure doesn't follow REST conventions");
            }

            // Display any errors
            if !result.stats.errors.is_empty() {
                println!();
                println!("⚠️  Errors encountered:");
                for (i, error) in result.stats.errors.iter().enumerate().take(5) {
                    println!("  {}. {}", i + 1, error);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Crawling failed: {}", e);
            return Err(e);
        }
    }

    println!();
    println!("💡 Try crawling APIs that use HAL or JSON API formats for better results!");
    println!("   Examples of good APIs to crawl:");
    println!("   - APIs using HAL (Hypertext Application Language)");
    println!("   - APIs following JSON API specification");
    println!("   - REST APIs with embedded link objects");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crawler_creation() {
        let config = CrawlerConfig::new().max_depth(2).max_concurrent_requests(3);

        let crawler = ApiCrawler::new(config);
        assert!(crawler.is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = CrawlerConfig::new()
            .max_depth(5)
            .max_concurrent_requests(20)
            .timeout_seconds(60)
            .allow_domain("example.com".to_string())
            .add_header("Accept".to_string(), "application/json".to_string());

        assert_eq!(config.max_depth, 5);
        assert_eq!(config.max_concurrent_requests, 20);
        assert_eq!(config.timeout_seconds, 60);
        assert!(config.allowed_domains.contains("example.com"));
        assert!(config.headers.contains_key("Accept"));
    }
}
