//! Tree Output Format Example
//!
//! This example demonstrates the new compact tree output format where
//! endpoints are organized in a hierarchical tree structure with all
//! related information in one block, eliminating redundancy.

use api_crawler::output::{OutputConfig, OutputFormat, serialize_result};
use api_crawler::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🌲 Tree Output Format Example");
    println!("=============================");
    println!("This example shows the new compact tree format where each");
    println!("endpoint contains all its children in one organized block.\n");

    // Create a mock crawl result to demonstrate tree structure
    let result = create_mock_tree_result();

    println!("📋 Mock API Structure:");
    println!("  Root: https://api.example.com");
    println!("  ├─ /users");
    println!("  │  ├─ /users/1");
    println!("  │  │  ├─ /users/1/posts");
    println!("  │  │  └─ /users/1/profile");
    println!("  │  └─ /users/search");
    println!("  ├─ /posts");
    println!("  │  ├─ /posts/123");
    println!("  │  │  ├─ /posts/123/comments");
    println!("  │  │  └─ /posts/123/likes");
    println!("  │  └─ /posts/recent");
    println!("  └─ /categories");
    println!("     └─ /categories/tech");
    println!();

    // Demonstrate different output formats
    demonstrate_tree_format(&result).await?;

    println!("🎯 Key Benefits of Tree Format:");
    println!("  • All endpoint information in one organized block");
    println!("  • Eliminates redundant parent-child references");
    println!("  • Smaller file size due to reduced repetition");
    println!("  • Natural tree visualization");
    println!("  • Easy to navigate programmatically");
    println!("  • Perfect for generating documentation");

    Ok(())
}

async fn demonstrate_tree_format(result: &CrawlResult) -> Result<()> {
    println!("📄 Tree Format Output:");
    println!("{}", "=".repeat(25));

    let tree_config = OutputConfig {
        format: OutputFormat::Tree,
        include_stats: true,
        include_config: false,
        hierarchical: false,
    };

    let tree_json = serialize_result(result, &tree_config)?;
    let parsed: serde_json::Value = serde_json::from_str(&tree_json)?;
    println!("{}", serde_json::to_string_pretty(&parsed)?);

    println!("\n💡 Notice how:");
    println!("  • Each endpoint contains its children inline");
    println!("  • No repetitive parent_url references");
    println!("  • Clean hierarchical structure");
    println!("  • Reduced file size compared to flat format");

    Ok(())
}

fn create_mock_tree_result() -> CrawlResult {
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

    let users_search_endpoint =
        ApiEndpoint::new("https://api.example.com/users/search".to_string(), 2)
            .with_rel(Some("search".to_string()))
            .with_parent(Some("https://api.example.com/users".to_string()));

    // User 1 sub-endpoints (depth 3)
    let user1_posts_endpoint =
        ApiEndpoint::new("https://api.example.com/users/1/posts".to_string(), 3)
            .with_rel(Some("user-posts".to_string()))
            .with_parent(Some("https://api.example.com/users/1".to_string()));

    let user1_profile_endpoint =
        ApiEndpoint::new("https://api.example.com/users/1/profile".to_string(), 3)
            .with_rel(Some("profile".to_string()))
            .with_parent(Some("https://api.example.com/users/1".to_string()));

    // Posts sub-endpoints (depth 2)
    let post123_endpoint = ApiEndpoint::new("https://api.example.com/posts/123".to_string(), 2)
        .with_rel(Some("post".to_string()))
        .with_parent(Some("https://api.example.com/posts".to_string()));

    let posts_recent_endpoint =
        ApiEndpoint::new("https://api.example.com/posts/recent".to_string(), 2)
            .with_rel(Some("recent".to_string()))
            .with_parent(Some("https://api.example.com/posts".to_string()));

    // Post 123 sub-endpoints (depth 3)
    let post123_comments_endpoint =
        ApiEndpoint::new("https://api.example.com/posts/123/comments".to_string(), 3)
            .with_rel(Some("comments".to_string()))
            .with_parent(Some("https://api.example.com/posts/123".to_string()));

    let post123_likes_endpoint =
        ApiEndpoint::new("https://api.example.com/posts/123/likes".to_string(), 3)
            .with_rel(Some("likes".to_string()))
            .with_parent(Some("https://api.example.com/posts/123".to_string()));

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
    result.add_endpoint(users_search_endpoint);
    result.add_endpoint(user1_posts_endpoint);
    result.add_endpoint(user1_profile_endpoint);
    result.add_endpoint(post123_endpoint);
    result.add_endpoint(posts_recent_endpoint);
    result.add_endpoint(post123_comments_endpoint);
    result.add_endpoint(post123_likes_endpoint);
    result.add_endpoint(tech_category_endpoint);

    // Update stats
    result.stats.urls_processed = 6; // Root + main endpoints
    result.stats.successful_requests = 6;
    result.stats.max_depth_reached = 3;
    result.stats.total_time_ms = 1850;

    result.complete();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_tree_result_creation() {
        let result = create_mock_tree_result();

        assert_eq!(result.start_url, "https://api.example.com");
        assert_eq!(result.endpoints.len(), 12);
        assert_eq!(result.stats.max_depth_reached, 3);
        assert!(!result.url_mappings.is_empty());
    }

    #[test]
    fn test_tree_structure_depth() {
        let result = create_mock_tree_result();

        // Verify depth distribution
        let depth1_count = result.endpoints_at_depth(1).len();
        let depth2_count = result.endpoints_at_depth(2).len();
        let depth3_count = result.endpoints_at_depth(3).len();

        assert_eq!(depth1_count, 3); // users, posts, categories
        assert_eq!(depth2_count, 5); // user/1, users/search, posts/123, posts/recent, categories/tech
        assert_eq!(depth3_count, 4); // users/1/posts, users/1/profile, posts/123/comments, posts/123/likes
    }

    #[tokio::test]
    async fn test_tree_serialization() {
        let result = create_mock_tree_result();

        let config = OutputConfig {
            format: OutputFormat::Tree,
            include_stats: false,
            include_config: false,
            hierarchical: false,
        };

        let json = serialize_result(&result, &config).unwrap();

        assert!(json.contains("api_tree"));
        assert!(json.contains("children"));
        assert!(json.contains("https://api.example.com/users"));
    }

    #[test]
    fn test_parent_child_relationships() {
        let result = create_mock_tree_result();

        // Verify some key parent-child relationships
        let users_children = result.url_mappings.get("https://api.example.com/users");
        assert!(users_children.is_some());
        assert_eq!(users_children.unwrap().len(), 2); // user/1 and users/search

        let user1_children = result.url_mappings.get("https://api.example.com/users/1");
        assert!(user1_children.is_some());
        assert_eq!(user1_children.unwrap().len(), 2); // posts and profile

        let posts_children = result.url_mappings.get("https://api.example.com/posts");
        assert!(posts_children.is_some());
        assert_eq!(posts_children.unwrap().len(), 2); // post/123 and recent
    }
}
