//! Hierarchical Output Example
//!
//! This example demonstrates the hierarchical output format where endpoints
//! are structured under their parent URLs in the JSON output.

use api_crawler::output::{
    OutputConfig, OutputFormat, print_hierarchical_summary, serialize_result,
};
use api_crawler::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🌳 Hierarchical Output Format Example");
    println!("=====================================");
    println!("This example shows how endpoints can be structured");
    println!("under their parent URLs in the output JSON.\n");

    // Create a mock crawl result to demonstrate hierarchical structure
    let result = create_mock_hierarchical_result();

    println!("📋 Mock API Structure:");
    println!("  Root: https://api.example.com");
    println!("  ├─ /users (depth 1)");
    println!("  │  ├─ /users/1 (depth 2)");
    println!("  │  ├─ /users/1/posts (depth 2)");
    println!("  │  └─ /users/1/profile (depth 2)");
    println!("  ├─ /posts (depth 1)");
    println!("  │  ├─ /posts/1 (depth 2)");
    println!("  │  └─ /posts/1/comments (depth 2)");
    println!("  └─ /categories (depth 1)");
    println!("     └─ /categories/tech (depth 2)");
    println!();

    // Demonstrate different output formats
    demonstrate_formats(&result).await?;

    // Show hierarchical summary
    print_hierarchical_summary(&result);

    println!("💡 Key Benefits of Hierarchical Format:");
    println!("  • Easier to understand parent-child relationships");
    println!("  • Better visualization of API structure");
    println!("  • Simpler navigation through the discovered endpoints");
    println!("  • More intuitive for API documentation generation");

    Ok(())
}

async fn demonstrate_formats(result: &CrawlResult) -> Result<()> {
    println!("📄 Output Format Comparison:");
    println!("{}", "=".repeat(30));

    // 1. Standard flat format
    println!("\n1️⃣  Standard Format (flat structure):");
    println!("{}", "-".repeat(45));
    let standard_config = OutputConfig {
        format: OutputFormat::CompactJson,
        include_stats: false,
        include_config: false,
        hierarchical: false,
    };

    let standard_json = serialize_result(result, &standard_config)?;
    let parsed: serde_json::Value = serde_json::from_str(&standard_json)?;
    println!("{}", serde_json::to_string_pretty(&parsed)?);

    // 2. Hierarchical format
    println!("\n2️⃣  Hierarchical Format (nested structure):");
    println!("{}", "-".repeat(50));
    let hierarchical_config = OutputConfig {
        format: OutputFormat::Hierarchical,
        include_stats: false,
        include_config: false,
        hierarchical: true,
    };

    let hierarchical_json = serialize_result(result, &hierarchical_config)?;
    let parsed: serde_json::Value = serde_json::from_str(&hierarchical_json)?;
    println!("{}", serde_json::to_string_pretty(&parsed)?);

    Ok(())
}

fn create_mock_hierarchical_result() -> CrawlResult {
    let config = CrawlerConfig::default();
    let mut result = CrawlResult::new("https://api.example.com".to_string(), &config);

    // Root level endpoints (depth 1)
    let users_endpoint = ApiEndpoint::new("https://api.example.com/users".to_string(), 1)
        .with_rel(Some("users".to_string()))
        .with_parent(Some("https://api.example.com".to_string()));

    let posts_endpoint = ApiEndpoint::new("https://api.example.com/posts".to_string(), 1)
        .with_rel(Some("posts".to_string()))
        .with_parent(Some("https://api.example.com".to_string()));

    let categories_endpoint = ApiEndpoint::new("https://api.example.com/categories".to_string(), 1)
        .with_rel(Some("categories".to_string()))
        .with_parent(Some("https://api.example.com".to_string()));

    // Users sub-endpoints (depth 2)
    let user1_endpoint = ApiEndpoint::new("https://api.example.com/users/1".to_string(), 2)
        .with_rel(Some("user".to_string()))
        .with_parent(Some("https://api.example.com/users".to_string()));

    let user1_posts_endpoint =
        ApiEndpoint::new("https://api.example.com/users/1/posts".to_string(), 2)
            .with_rel(Some("user-posts".to_string()))
            .with_parent(Some("https://api.example.com/users".to_string()));

    let user1_profile_endpoint =
        ApiEndpoint::new("https://api.example.com/users/1/profile".to_string(), 2)
            .with_rel(Some("profile".to_string()))
            .with_parent(Some("https://api.example.com/users".to_string()));

    // Posts sub-endpoints (depth 2)
    let post1_endpoint = ApiEndpoint::new("https://api.example.com/posts/1".to_string(), 2)
        .with_rel(Some("post".to_string()))
        .with_parent(Some("https://api.example.com/posts".to_string()));

    let post1_comments_endpoint =
        ApiEndpoint::new("https://api.example.com/posts/1/comments".to_string(), 2)
            .with_rel(Some("comments".to_string()))
            .with_parent(Some("https://api.example.com/posts".to_string()));

    // Categories sub-endpoints (depth 2)
    let tech_category_endpoint =
        ApiEndpoint::new("https://api.example.com/categories/tech".to_string(), 2)
            .with_rel(Some("category".to_string()))
            .with_parent(Some("https://api.example.com/categories".to_string()));

    // Add all endpoints to result
    result.add_endpoint(users_endpoint);
    result.add_endpoint(posts_endpoint);
    result.add_endpoint(categories_endpoint);
    result.add_endpoint(user1_endpoint);
    result.add_endpoint(user1_posts_endpoint);
    result.add_endpoint(user1_profile_endpoint);
    result.add_endpoint(post1_endpoint);
    result.add_endpoint(post1_comments_endpoint);
    result.add_endpoint(tech_category_endpoint);

    // Update stats
    result.stats.urls_processed = 4; // Root + 3 main endpoints
    result.stats.successful_requests = 4;
    result.stats.max_depth_reached = 2;

    result.complete();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_result_creation() {
        let result = create_mock_hierarchical_result();

        assert_eq!(result.start_url, "https://api.example.com");
        assert_eq!(result.endpoints.len(), 9);
        assert_eq!(result.stats.max_depth_reached, 2);
        assert!(!result.url_mappings.is_empty());
    }

    #[test]
    fn test_hierarchical_structure() {
        let result = create_mock_hierarchical_result();

        // Check that we have parent-child relationships
        assert!(result.url_mappings.contains_key("https://api.example.com"));
        assert!(
            result
                .url_mappings
                .contains_key("https://api.example.com/users")
        );
        assert!(
            result
                .url_mappings
                .contains_key("https://api.example.com/posts")
        );

        // Verify depth distribution
        let depth1_count = result.endpoints_at_depth(1).len();
        let depth2_count = result.endpoints_at_depth(2).len();

        assert_eq!(depth1_count, 3); // users, posts, categories
        assert_eq!(depth2_count, 6); // all the sub-endpoints
    }

    #[tokio::test]
    async fn test_hierarchical_serialization() {
        let result = create_mock_hierarchical_result();

        let config = OutputConfig {
            format: OutputFormat::Hierarchical,
            include_stats: false,
            include_config: false,
            hierarchical: true,
        };

        let json = serialize_result(&result, &config).unwrap();

        assert!(json.contains("endpoint_hierarchy"));
        assert!(json.contains("https://api.example.com/users"));
        assert!(json.contains("summary"));
    }
}
