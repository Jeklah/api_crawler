//! Output handling for API crawler results

use crate::error::{CrawlerError, Result};
use crate::types::CrawlResult;
use serde_json;
use std::fs;
use std::path::Path;
use tracing::info;

/// Output format options
#[derive(Debug, Clone)]
pub enum OutputFormat {
    /// Pretty-printed JSON
    PrettyJson,
    /// Compact JSON
    CompactJson,
    /// Hierarchical structure with endpoints nested under parent URLs
    Hierarchical,
    /// Compact tree structure with all endpoint info in one block
    Tree,
}

/// Output configuration
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Format for the output
    pub format: OutputFormat,

    /// Whether to include statistics in output
    pub include_stats: bool,

    /// Whether to include configuration snapshot
    pub include_config: bool,

    /// Whether to use hierarchical structure (endpoints nested under parents)
    pub hierarchical: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::PrettyJson,
            include_stats: true,
            include_config: true,
            hierarchical: false,
        }
    }
}

/// Save crawl results to a JSON file
pub fn save_results_to_file<P: AsRef<Path>>(
    result: &CrawlResult,
    file_path: P,
    config: Option<OutputConfig>,
) -> Result<()> {
    let config = config.unwrap_or_default();
    let path = file_path.as_ref();

    info!("Saving results to: {}", path.display());

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let json_string = serialize_result(result, &config)?;
    fs::write(path, json_string)?;

    info!("Results saved successfully to: {}", path.display());
    Ok(())
}

/// Serialize crawl results to JSON string
pub fn serialize_result(result: &CrawlResult, config: &OutputConfig) -> Result<String> {
    match config.format {
        OutputFormat::Tree => serialize_tree_result(result, config),
        OutputFormat::Hierarchical => serialize_hierarchical_result(result, config),
        _ if config.hierarchical => serialize_hierarchical_result(result, config),
        OutputFormat::PrettyJson | OutputFormat::CompactJson => {
            let mut result_copy = result.clone();

            // Filter out unwanted fields based on config
            if !config.include_stats {
                result_copy.stats = Default::default();
            }

            if !config.include_config {
                result_copy.config_snapshot = String::new();
            }

            match config.format {
                OutputFormat::PrettyJson | OutputFormat::Hierarchical | OutputFormat::Tree => {
                    serde_json::to_string_pretty(&result_copy).map_err(CrawlerError::from)
                }
                OutputFormat::CompactJson => {
                    serde_json::to_string(&result_copy).map_err(CrawlerError::from)
                }
            }
        }
    }
}

/// Serialize crawl results in hierarchical format
fn serialize_hierarchical_result(result: &CrawlResult, config: &OutputConfig) -> Result<String> {
    use indexmap::IndexMap;
    use serde_json::{Value, json};

    let mut hierarchical_structure = IndexMap::new();

    // Build hierarchical structure
    for endpoint in &result.endpoints {
        let parent_key = endpoint.parent_url.as_deref().unwrap_or(&result.start_url);

        // Create endpoint object - only include non-null/non-empty fields
        let mut endpoint_obj = IndexMap::new();
        endpoint_obj.insert("href".to_string(), Value::String(endpoint.href.clone()));

        // Only include optional fields if they have values
        if let Some(ref rel) = endpoint.rel {
            endpoint_obj.insert("rel".to_string(), Value::String(rel.clone()));
        }
        if let Some(ref method) = endpoint.method {
            endpoint_obj.insert("method".to_string(), Value::String(method.clone()));
        }
        if let Some(ref content_type) = endpoint.r#type {
            endpoint_obj.insert("type".to_string(), Value::String(content_type.clone()));
        }
        if let Some(ref title) = endpoint.title {
            endpoint_obj.insert("title".to_string(), Value::String(title.clone()));
        }

        endpoint_obj.insert("depth".to_string(), Value::Number(endpoint.depth.into()));

        // Only include metadata if it's not empty
        if !endpoint.metadata.is_empty() {
            endpoint_obj.insert("metadata".to_string(), json!(endpoint.metadata));
        }

        // Add to hierarchical structure
        let children = hierarchical_structure
            .entry(parent_key.to_string())
            .or_insert_with(|| Value::Array(Vec::new()));

        if let Value::Array(children_array) = children {
            children_array.push(Value::Object(endpoint_obj.into_iter().collect()));
        }
    }

    // Build final output structure
    let mut output = IndexMap::new();
    output.insert(
        "start_url".to_string(),
        Value::String(result.start_url.clone()),
    );
    output.insert(
        "endpoint_hierarchy".to_string(),
        Value::Object(hierarchical_structure.into_iter().collect()),
    );

    // Add summary information
    let mut summary = IndexMap::new();
    summary.insert(
        "total_endpoints".to_string(),
        Value::Number(result.endpoints.len().into()),
    );
    summary.insert(
        "unique_parents".to_string(),
        Value::Number(result.url_mappings.len().into()),
    );
    summary.insert(
        "discovered_domains".to_string(),
        Value::Number(result.discovered_domains().len().into()),
    );
    output.insert(
        "summary".to_string(),
        Value::Object(summary.into_iter().collect()),
    );

    if config.include_stats {
        output.insert("stats".to_string(), json!(result.stats));
    }

    output.insert(
        "started_at".to_string(),
        Value::String(result.started_at.to_rfc3339()),
    );
    output.insert(
        "completed_at".to_string(),
        Value::String(result.completed_at.to_rfc3339()),
    );

    if config.include_config {
        output.insert(
            "config_snapshot".to_string(),
            Value::String(result.config_snapshot.clone()),
        );
    }

    // Serialize based on format preference
    let final_json = Value::Object(output.into_iter().collect());
    match config.format {
        OutputFormat::CompactJson => serde_json::to_string(&final_json).map_err(CrawlerError::from),
        OutputFormat::PrettyJson | OutputFormat::Hierarchical | OutputFormat::Tree => {
            serde_json::to_string_pretty(&final_json).map_err(CrawlerError::from)
        }
    }
}

/// Serialize crawl results in compact tree format
fn serialize_tree_result(result: &CrawlResult, config: &OutputConfig) -> Result<String> {
    use crate::types::ApiEndpoint;
    use indexmap::IndexMap;
    use serde_json::{Value, json};
    use std::collections::{HashMap, HashSet};

    // Safety check for empty results
    if result.endpoints.is_empty() {
        let mut output = IndexMap::new();
        output.insert(
            "start_url".to_string(),
            Value::String(result.start_url.clone()),
        );
        output.insert("api_tree".to_string(), Value::Null);

        let mut summary = IndexMap::new();
        summary.insert("total_endpoints".to_string(), Value::Number(0.into()));
        summary.insert("max_depth".to_string(), Value::Number(0.into()));
        summary.insert("discovered_domains".to_string(), Value::Number(0.into()));
        output.insert(
            "summary".to_string(),
            Value::Object(summary.into_iter().collect()),
        );

        if config.include_stats {
            output.insert("stats".to_string(), json!(result.stats));
        }

        output.insert(
            "started_at".to_string(),
            Value::String(result.started_at.to_rfc3339()),
        );
        output.insert(
            "completed_at".to_string(),
            Value::String(result.completed_at.to_rfc3339()),
        );

        if config.include_config {
            output.insert(
                "config_snapshot".to_string(),
                Value::String(result.config_snapshot.clone()),
            );
        }

        let json_value = Value::Object(output.into_iter().collect());
        return serde_json::to_string_pretty(&json_value).map_err(CrawlerError::from);
    }

    // Deduplicate endpoints by href and keep the one with most metadata
    let mut unique_endpoints: HashMap<String, ApiEndpoint> = HashMap::new();
    for endpoint in &result.endpoints {
        let existing = unique_endpoints.get(&endpoint.href);
        match existing {
            Some(existing_endpoint) => {
                // Keep the endpoint with more metadata or the one that's not self-referential
                if endpoint.metadata.len() > existing_endpoint.metadata.len()
                    || (endpoint.rel.as_deref() != Some("self")
                        && existing_endpoint.rel.as_deref() == Some("self"))
                {
                    unique_endpoints.insert(endpoint.href.clone(), endpoint.clone());
                }
            }
            None => {
                unique_endpoints.insert(endpoint.href.clone(), endpoint.clone());
            }
        }
    }

    let endpoints: Vec<&ApiEndpoint> = unique_endpoints.values().collect();

    // Build a clean tree node structure where parent info appears before children
    fn build_tree_node(
        endpoint: &ApiEndpoint,
        all_endpoints: &[&ApiEndpoint],
        processed: &mut HashSet<String>,
    ) -> IndexMap<String, Value> {
        let mut node = IndexMap::new();

        // Extract name from URL (last path segment)
        let name = endpoint
            .href
            .split('/')
            .last()
            .unwrap_or(&endpoint.href)
            .to_string();

        // Use rel from metadata if available, otherwise use the rel field
        let rel = endpoint
            .metadata
            .get("rel")
            .and_then(|v| v.as_str())
            .or(endpoint.rel.as_deref())
            .unwrap_or("unknown");

        // Create endpoint info structure
        let mut endpoint_info = IndexMap::new();
        endpoint_info.insert("name".to_string(), Value::String(name));
        endpoint_info.insert("url".to_string(), Value::String(endpoint.href.clone()));
        endpoint_info.insert("rel".to_string(), Value::String(rel.to_string()));
        endpoint_info.insert("depth".to_string(), Value::Number(endpoint.depth.into()));

        // Add optional fields if present
        if let Some(ref method) = endpoint.method {
            endpoint_info.insert("method".to_string(), Value::String(method.clone()));
        }
        if let Some(ref content_type) = endpoint.r#type {
            endpoint_info.insert("type".to_string(), Value::String(content_type.clone()));
        }
        if let Some(ref title) = endpoint.title {
            endpoint_info.insert("title".to_string(), Value::String(title.clone()));
        }

        // Put endpoint info first
        node.insert(
            "api".to_string(),
            Value::Object(endpoint_info.into_iter().collect()),
        );

        // Find and sort children
        let mut children: Vec<&ApiEndpoint> = all_endpoints
            .iter()
            .filter(|e| {
                e.parent_url.as_ref() == Some(&endpoint.href)
                    && !processed.contains(&e.href)
                    && e.href != endpoint.href // Avoid self-reference
            })
            .cloned()
            .collect();

        // Sort children by depth first, then alphabetically by name
        children.sort_by(|a, b| {
            a.depth.cmp(&b.depth).then_with(|| {
                let name_a = a.href.split('/').last().unwrap_or("");
                let name_b = b.href.split('/').last().unwrap_or("");
                name_a.cmp(name_b)
            })
        });

        // Add children after the parent endpoint info
        if !children.is_empty() {
            let mut child_nodes = Vec::new();
            for child in children {
                if !processed.contains(&child.href) {
                    processed.insert(child.href.clone());
                    let child_node = build_tree_node(child, all_endpoints, processed);
                    child_nodes.push(Value::Object(child_node.into_iter().collect()));
                }
            }
            if !child_nodes.is_empty() {
                node.insert("children".to_string(), Value::Array(child_nodes));
            }
        }

        node
    }

    // Find root endpoint - prioritize self-referential endpoints at start_url
    let root_endpoint = endpoints
        .iter()
        .find(|e| {
            let matches = e.href == result.start_url
                && e.parent_url.as_ref() == Some(&result.start_url)
                && e.rel.as_deref() == Some("self");

            matches
        })
        .or_else(|| {
            let found = endpoints.iter().find(|e| e.href == result.start_url);

            found
        })
        .or_else(|| {
            let found = endpoints.iter().find(|e| e.depth == 0);

            found
        })
        .or_else(|| endpoints.first())
        .map(|e| (*e).clone());

    let mut processed = HashSet::new();

    let api_tree = if let Some(root) = root_endpoint {
        // Extract root endpoint info
        let name = root
            .href
            .split('/')
            .last()
            .unwrap_or(&root.href)
            .to_string();
        let rel = root
            .metadata
            .get("rel")
            .and_then(|v| v.as_str())
            .or(root.rel.as_deref())
            .unwrap_or("self");

        // Mark root as processed
        processed.insert(root.href.clone());

        // Build children
        let mut children: Vec<&ApiEndpoint> = endpoints
            .iter()
            .filter(|e| {
                e.parent_url.as_ref() == Some(&root.href)
                    && !processed.contains(&e.href)
                    && e.href != root.href // Avoid self-reference
            })
            .cloned()
            .collect();

        // Sort children by depth first, then alphabetically by name
        children.sort_by(|a, b| {
            a.depth.cmp(&b.depth).then_with(|| {
                let name_a = a.href.split('/').last().unwrap_or("");
                let name_b = b.href.split('/').last().unwrap_or("");
                name_a.cmp(name_b)
            })
        });

        let mut child_nodes = Vec::new();
        for child in children {
            if !processed.contains(&child.href) {
                processed.insert(child.href.clone());
                let child_node = build_tree_node(child, &endpoints, &mut processed);
                child_nodes.push(Value::Object(child_node.into_iter().collect()));
            }
        }

        // Build JSON structure manually to guarantee field order
        use serde_json::Map;
        let mut root_object = Map::new();

        // Insert endpoint info FIRST
        let mut endpoint_info = Map::new();
        endpoint_info.insert("name".to_string(), Value::String(name));
        endpoint_info.insert("url".to_string(), Value::String(root.href.clone()));
        endpoint_info.insert("rel".to_string(), Value::String(rel.to_string()));
        endpoint_info.insert("depth".to_string(), Value::Number(root.depth.into()));

        if let Some(ref method) = root.method {
            endpoint_info.insert("method".to_string(), Value::String(method.clone()));
        }
        if let Some(ref content_type) = root.r#type {
            endpoint_info.insert("type".to_string(), Value::String(content_type.clone()));
        }
        if let Some(ref title) = root.title {
            endpoint_info.insert("title".to_string(), Value::String(title.clone()));
        }

        root_object.insert("api".to_string(), Value::Object(endpoint_info));

        // Insert children SECOND (only if not empty)
        if !child_nodes.is_empty() {
            root_object.insert("children".to_string(), Value::Array(child_nodes));
        }

        Value::Object(root_object)
    } else {
        Value::Null
    };

    // Build final output structure
    let mut output = IndexMap::new();
    output.insert(
        "start_url".to_string(),
        Value::String(result.start_url.clone()),
    );
    output.insert("api_tree".to_string(), api_tree);

    // Add summary
    let mut summary = IndexMap::new();
    summary.insert(
        "total_endpoints".to_string(),
        Value::Number(unique_endpoints.len().into()),
    );
    summary.insert(
        "max_depth".to_string(),
        Value::Number(endpoints.iter().map(|e| e.depth).max().unwrap_or(0).into()),
    );
    summary.insert(
        "discovered_domains".to_string(),
        Value::Number(result.discovered_domains().len().into()),
    );
    output.insert(
        "summary".to_string(),
        Value::Object(summary.into_iter().collect()),
    );

    if config.include_stats {
        output.insert("stats".to_string(), json!(result.stats));
    }

    output.insert(
        "started_at".to_string(),
        Value::String(result.started_at.to_rfc3339()),
    );
    output.insert(
        "completed_at".to_string(),
        Value::String(result.completed_at.to_rfc3339()),
    );

    if config.include_config {
        output.insert(
            "config_snapshot".to_string(),
            Value::String(result.config_snapshot.clone()),
        );
    }

    let json_value = Value::Object(output.into_iter().collect());

    // Final safety check before serialization
    match serde_json::to_string_pretty(&json_value) {
        Ok(json_string) => {
            tracing::debug!(
                "Successfully serialized tree format with {} characters",
                json_string.len()
            );
            Ok(json_string)
        }
        Err(e) => {
            tracing::error!("Failed to serialize tree format: {}", e);
            Err(CrawlerError::from(e))
        }
    }
}

/// Print a summary of the crawl results to stdout
pub fn print_summary(result: &CrawlResult) {
    println!("\n🕷️  API Crawl Summary");
    println!("═══════════════════");
    println!("Start URL: {}", result.start_url);
    println!(
        "Started at: {}",
        result.started_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!(
        "Completed at: {}",
        result.completed_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!();

    // Statistics
    println!("📊 Statistics:");
    println!("  • URLs processed: {}", result.stats.urls_processed);
    println!(
        "  • Successful requests: {}",
        result.stats.successful_requests
    );
    println!("  • Failed requests: {}", result.stats.failed_requests);
    println!("  • URLs skipped: {}", result.stats.urls_skipped);
    println!("  • Max depth reached: {}", result.stats.max_depth_reached);
    println!("  • Total time: {}ms", result.stats.total_time_ms);
    println!();

    // Endpoints
    println!("🔗 Discovered Endpoints:");
    println!("  • Total endpoints: {}", result.endpoints.len());
    println!("  • Unique domains: {}", result.discovered_domains().len());
    println!("  • Parent URLs: {}", result.url_mappings.len());

    // Breakdown by depth
    let mut depth_counts = std::collections::HashMap::new();
    for endpoint in &result.endpoints {
        *depth_counts.entry(endpoint.depth).or_insert(0) += 1;
    }

    println!("  • Endpoints by depth:");
    let mut depths: Vec<_> = depth_counts.keys().collect();
    depths.sort();
    for depth in depths {
        println!("    - Depth {}: {} endpoints", depth, depth_counts[depth]);
    }

    // Show hierarchical breakdown
    if !result.url_mappings.is_empty() {
        println!();
        println!("🌳 Hierarchical Structure:");
        let mut parents: Vec<_> = result.url_mappings.keys().collect();
        parents.sort();
        for (i, parent) in parents.iter().enumerate().take(5) {
            let children = &result.url_mappings[*parent];
            println!("  {}. {} → {} endpoints", i + 1, parent, children.len());
            for (_j, child) in children.iter().enumerate().take(3) {
                println!("     └─ {}", child.href);
            }
            if children.len() > 3 {
                println!("     └─ ... and {} more", children.len() - 3);
            }
        }
        if result.url_mappings.len() > 5 {
            println!(
                "  ... and {} more parent URLs",
                result.url_mappings.len() - 5
            );
        }
    }

    // Domains
    if !result.discovered_domains().is_empty() {
        println!();
        println!("🌐 Discovered Domains:");
        let mut domains: Vec<_> = result.discovered_domains().into_iter().collect();
        domains.sort();
        for domain in domains {
            let domain_endpoints = result
                .endpoints
                .iter()
                .filter(|e| e.href.contains(&domain))
                .count();
            println!("  • {}: {} endpoints", domain, domain_endpoints);
        }
    }

    // Errors
    if !result.stats.errors.is_empty() {
        println!();
        println!("⚠️  Errors ({}):", result.stats.errors.len());
        for (i, error) in result.stats.errors.iter().enumerate().take(5) {
            println!("  {}. {}", i + 1, error);
        }
        if result.stats.errors.len() > 5 {
            println!("  ... and {} more errors", result.stats.errors.len() - 5);
        }
    }

    println!();
}

/// Print hierarchical structure of endpoints
pub fn print_hierarchical_summary(result: &CrawlResult) {
    println!("\n🌳 Hierarchical API Structure");
    println!("════════════════════════════");

    if result.url_mappings.is_empty() {
        println!("No parent-child relationships discovered.");
        return;
    }

    let mut parents: Vec<_> = result.url_mappings.keys().collect();
    parents.sort();

    for parent in parents {
        let children = &result.url_mappings[parent];
        println!("\n📁 {}", parent);

        for child in children {
            println!("  ├─ {} (depth: {})", child.href, child.depth);
            if let Some(ref rel) = child.rel {
                println!("  │  └─ rel: {}", rel);
            }
        }
    }
    println!();
}

/// Print detailed endpoint information
pub fn print_endpoints_detailed(result: &CrawlResult, max_endpoints: Option<usize>) {
    let max = max_endpoints.unwrap_or(result.endpoints.len());

    println!("\n🔍 Detailed Endpoint Information");
    println!("══════════════════════════════");

    for (i, endpoint) in result.endpoints.iter().enumerate().take(max) {
        println!("{}. {}", i + 1, endpoint.href);

        if let Some(ref rel) = endpoint.rel {
            println!("   Relation: {}", rel);
        }

        if let Some(ref method) = endpoint.method {
            println!("   Method: {}", method);
        }

        if let Some(ref content_type) = endpoint.r#type {
            println!("   Type: {}", content_type);
        }

        if let Some(ref title) = endpoint.title {
            println!("   Title: {}", title);
        }

        println!("   Depth: {}", endpoint.depth);

        if let Some(ref parent) = endpoint.parent_url {
            println!("   Parent: {}", parent);
        }

        if !endpoint.metadata.is_empty() {
            println!("   Metadata:");
            for (key, value) in &endpoint.metadata {
                println!("     {}: {}", key, value);
            }
        }

        if i < max - 1 {
            println!();
        }
    }

    if result.endpoints.len() > max {
        println!("\n... and {} more endpoints", result.endpoints.len() - max);
    }
}

/// Generate a simple text report
pub fn generate_text_report(result: &CrawlResult) -> String {
    let mut report = String::new();

    report.push_str(&format!("API Crawl Report\n"));
    report.push_str(&format!("================\n\n"));

    report.push_str(&format!("Start URL: {}\n", result.start_url));
    report.push_str(&format!("Duration: {}ms\n", result.stats.total_time_ms));
    report.push_str(&format!(
        "URLs Processed: {}\n",
        result.stats.urls_processed
    ));
    report.push_str(&format!("Endpoints Found: {}\n", result.endpoints.len()));
    report.push_str(&format!(
        "Success Rate: {:.1}%\n\n",
        if result.stats.urls_processed > 0 {
            (result.stats.successful_requests as f64 / result.stats.urls_processed as f64) * 100.0
        } else {
            0.0
        }
    ));

    report.push_str("Endpoints by Relation Type:\n");
    report.push_str("---------------------------\n");

    let mut rel_counts = std::collections::HashMap::new();
    for endpoint in &result.endpoints {
        let rel = endpoint.rel.as_deref().unwrap_or("(none)");
        *rel_counts.entry(rel).or_insert(0) += 1;
    }

    let mut rels: Vec<_> = rel_counts.iter().collect();
    rels.sort_by(|a, b| b.1.cmp(a.1));

    for (rel, count) in rels {
        report.push_str(&format!("  {}: {}\n", rel, count));
    }

    if !result.stats.errors.is_empty() {
        report.push_str("\nErrors:\n");
        report.push_str("-------\n");
        for error in &result.stats.errors {
            report.push_str(&format!("  • {}\n", error));
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ApiEndpoint, CrawlerConfig};
    use tempfile::NamedTempFile;

    #[test]
    fn test_serialize_result() {
        let mut result =
            CrawlResult::new("http://example.com".to_string(), &CrawlerConfig::default());
        result
            .endpoints
            .push(ApiEndpoint::new("http://example.com/test".to_string(), 1));

        let config = OutputConfig::default();
        let json = serialize_result(&result, &config).unwrap();

        assert!(json.contains("http://example.com"));
        assert!(json.contains("endpoints"));
    }

    #[test]
    fn test_save_results_to_file() {
        let mut result =
            CrawlResult::new("http://example.com".to_string(), &CrawlerConfig::default());
        result
            .endpoints
            .push(ApiEndpoint::new("http://example.com/test".to_string(), 1));

        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path();

        save_results_to_file(&result, file_path, None).unwrap();

        let content = std::fs::read_to_string(file_path).unwrap();
        assert!(content.contains("http://example.com"));
    }

    #[test]
    fn test_generate_text_report() {
        let mut result =
            CrawlResult::new("http://example.com".to_string(), &CrawlerConfig::default());

        let endpoint = ApiEndpoint::new("http://example.com/test".to_string(), 1)
            .with_rel(Some("next".to_string()));
        result.endpoints.push(endpoint);

        result.stats.urls_processed = 1;
        result.stats.successful_requests = 1;

        let report = generate_text_report(&result);

        assert!(report.contains("API Crawl Report"));
        assert!(report.contains("http://example.com"));
        assert!(report.contains("next: 1"));
    }

    #[test]
    fn test_hierarchical_serialization() {
        let mut result =
            CrawlResult::new("http://example.com".to_string(), &CrawlerConfig::default());

        let endpoint1 = ApiEndpoint::new("http://example.com/users".to_string(), 1)
            .with_rel(Some("users".to_string()))
            .with_parent(Some("http://example.com".to_string()));

        let endpoint2 = ApiEndpoint::new("http://example.com/posts".to_string(), 1)
            .with_rel(Some("posts".to_string()))
            .with_parent(Some("http://example.com".to_string()));

        result.endpoints.push(endpoint1);
        result.endpoints.push(endpoint2);
        result
            .url_mappings
            .insert("http://example.com".to_string(), result.endpoints.clone());

        let config = OutputConfig {
            format: OutputFormat::Hierarchical,
            include_stats: true,
            include_config: false,
            hierarchical: true,
        };

        let json = serialize_result(&result, &config).unwrap();
        assert!(json.contains("endpoint_hierarchy"));
        assert!(json.contains("http://example.com/users"));
        assert!(json.contains("http://example.com/posts"));
        assert!(json.contains("summary"));
    }

    #[test]
    fn test_clean_output_omits_null_fields() {
        let mut result =
            CrawlResult::new("http://example.com".to_string(), &CrawlerConfig::default());

        // Create endpoint with only some fields populated
        let endpoint = ApiEndpoint::new("http://example.com/test".to_string(), 1)
            .with_rel(Some("test".to_string()))
            .with_parent(Some("http://example.com".to_string()));
        // Note: method, type, title remain None, metadata remains empty

        result.endpoints.push(endpoint);

        let config = OutputConfig {
            format: OutputFormat::PrettyJson,
            include_stats: true,
            include_config: false,
            hierarchical: false,
        };

        let json = serialize_result(&result, &config).unwrap();

        // Should include fields with values
        assert!(json.contains("href"));
        assert!(json.contains("rel"));
        assert!(json.contains("depth"));
        assert!(json.contains("parent_url"));

        // Should NOT include null/empty fields
        assert!(!json.contains("method"));
        assert!(!json.contains("type"));
        assert!(!json.contains("title"));
        assert!(!json.contains("metadata"));
        assert!(!json.contains("config_snapshot"));

        // Stats with zero values should be omitted
        assert!(!json.contains("urls_processed"));
        assert!(!json.contains("successful_requests"));
        assert!(!json.contains("failed_requests"));
        assert!(!json.contains("urls_skipped"));
        assert!(!json.contains("max_depth_reached"));
        assert!(!json.contains("total_time_ms"));
        assert!(!json.contains("errors"));
    }

    #[test]
    fn test_hierarchical_output_omits_null_fields() {
        let mut result =
            CrawlResult::new("http://example.com".to_string(), &CrawlerConfig::default());

        // Create endpoint with only some fields populated
        let endpoint = ApiEndpoint::new("http://example.com/test".to_string(), 1)
            .with_rel(Some("test".to_string()))
            .with_parent(Some("http://example.com".to_string()));
        // Note: method, type, title remain None, metadata remains empty

        result.endpoints.push(endpoint);

        let config = OutputConfig {
            format: OutputFormat::Hierarchical,
            include_stats: false,
            include_config: false,
            hierarchical: true,
        };

        let json = serialize_result(&result, &config).unwrap();

        // Should include fields with values
        assert!(json.contains("href"));
        assert!(json.contains("rel"));
        assert!(json.contains("depth"));

        // Should NOT include null/empty fields in hierarchical format
        assert!(!json.contains("method"));
        assert!(!json.contains("type"));
        assert!(!json.contains("title"));
        assert!(!json.contains("metadata"));
        assert!(!json.contains("config_snapshot"));
        assert!(!json.contains("stats"));
    }

    #[test]
    fn test_tree_format_serialization() {
        let mut result =
            CrawlResult::new("http://example.com".to_string(), &CrawlerConfig::default());

        // Add a root endpoint
        let root_endpoint = ApiEndpoint::new("http://example.com".to_string(), 0)
            .with_rel(Some("self".to_string()));

        let parent_endpoint = ApiEndpoint::new("http://example.com/users".to_string(), 1)
            .with_rel(Some("users".to_string()))
            .with_parent(Some("http://example.com".to_string()));

        let child_endpoint = ApiEndpoint::new("http://example.com/users/1".to_string(), 2)
            .with_rel(Some("user".to_string()))
            .with_parent(Some("http://example.com/users".to_string()));

        result.endpoints.push(root_endpoint);
        result.endpoints.push(parent_endpoint);
        result.endpoints.push(child_endpoint);

        let config = OutputConfig {
            format: OutputFormat::Tree,
            include_stats: false,
            include_config: false,
            hierarchical: false,
        };

        let json = serialize_result(&result, &config).unwrap();
        assert!(json.contains("api_tree"));
        assert!(json.contains("api"));
        assert!(json.contains("children"));
        assert!(json.contains("http://example.com/users"));
        assert!(json.contains("\"name\":"));
        assert!(json.contains("\"url\":"));
        assert!(json.contains("\"rel\":"));
    }
}
