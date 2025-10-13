//! Test example demonstrating clean single-children tree structure
//!
//! This test shows how the tree format should create exactly one 'children'
//! object per endpoint, with proper parent-child relationships.

use api_crawler::output::{OutputConfig, OutputFormat, serialize_result};
use api_crawler::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Clean Tree Structure Test");
    println!("============================");
    println!("This test demonstrates the correct tree structure:");
    println!("• One 'children' array per parent");
    println!("• Clean parent-child relationships");
    println!("• No duplicate processing");
    println!();

    // Create test result with clear parent-child relationships
    let result = create_clean_test_structure();

    println!("📋 Expected Structure:");
    println!("http://qx-022160:8080/api/v1");
    println!("├── children: [generation, audio, analyser]");
    println!("│   ├── generation");
    println!("│   │   └── children: [text-to-speech, speech-enhancement]");
    println!("│   ├── audio");
    println!("│   │   └── children: [upload, process]");
    println!("│   └── analyser");
    println!("│       └── children: [sentiment, emotion]");
    println!();

    // Test tree format
    let tree_config = OutputConfig {
        format: OutputFormat::Tree,
        include_stats: false,
        include_config: false,
        hierarchical: false,
    };

    match serialize_result(&result, &tree_config) {
        Ok(json) => {
            // Save to test directory
            std::fs::create_dir_all("test").expect("Failed to create test directory");
            std::fs::write("test/clean_tree_structure.json", &json)
                .expect("Failed to write test file");

            println!("✅ Tree format generated successfully!");
            println!("📁 Saved to: test/clean_tree_structure.json");
            println!();

            // Parse and analyze structure
            let parsed: serde_json::Value =
                serde_json::from_str(&json).expect("Failed to parse generated JSON");

            analyze_tree_structure(&parsed);

            println!("🔍 Generated JSON:");
            println!("{}", json);
        }
        Err(e) => {
            println!("❌ Failed to generate tree format: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

fn create_clean_test_structure() -> CrawlResult {
    let config = CrawlerConfig::default();
    let mut result = CrawlResult::new("http://qx-022160:8080/api/v1".to_string(), &config);

    // Level 1: Direct children of /api/v1
    let generation = ApiEndpoint::new("http://qx-022160:8080/api/v1/generation".to_string(), 1)
        .with_rel(Some("generation".to_string()))
        .with_parent(Some("http://qx-022160:8080/api/v1".to_string()));

    let audio = ApiEndpoint::new("http://qx-022160:8080/api/v1/audio".to_string(), 1)
        .with_rel(Some("audio".to_string()))
        .with_parent(Some("http://qx-022160:8080/api/v1".to_string()));

    let analyser = ApiEndpoint::new("http://qx-022160:8080/api/v1/analyser".to_string(), 1)
        .with_rel(Some("analyser".to_string()))
        .with_parent(Some("http://qx-022160:8080/api/v1".to_string()));

    // Level 2: Children of generation
    let text_to_speech = ApiEndpoint::new(
        "http://qx-022160:8080/api/v1/generation/text-to-speech".to_string(),
        2,
    )
    .with_rel(Some("text-to-speech".to_string()))
    .with_parent(Some("http://qx-022160:8080/api/v1/generation".to_string()));

    let speech_enhancement = ApiEndpoint::new(
        "http://qx-022160:8080/api/v1/generation/speech-enhancement".to_string(),
        2,
    )
    .with_rel(Some("speech-enhancement".to_string()))
    .with_parent(Some("http://qx-022160:8080/api/v1/generation".to_string()));

    // Level 2: Children of audio
    let audio_upload = ApiEndpoint::new("http://qx-022160:8080/api/v1/audio/upload".to_string(), 2)
        .with_rel(Some("upload".to_string()))
        .with_parent(Some("http://qx-022160:8080/api/v1/audio".to_string()));

    let audio_process =
        ApiEndpoint::new("http://qx-022160:8080/api/v1/audio/process".to_string(), 2)
            .with_rel(Some("process".to_string()))
            .with_parent(Some("http://qx-022160:8080/api/v1/audio".to_string()));

    // Level 2: Children of analyser
    let sentiment = ApiEndpoint::new(
        "http://qx-022160:8080/api/v1/analyser/sentiment".to_string(),
        2,
    )
    .with_rel(Some("sentiment".to_string()))
    .with_parent(Some("http://qx-022160:8080/api/v1/analyser".to_string()));

    let emotion = ApiEndpoint::new(
        "http://qx-022160:8080/api/v1/analyser/emotion".to_string(),
        2,
    )
    .with_rel(Some("emotion".to_string()))
    .with_parent(Some("http://qx-022160:8080/api/v1/analyser".to_string()));

    // Add all endpoints to result
    result.add_endpoint(generation);
    result.add_endpoint(audio);
    result.add_endpoint(analyser);
    result.add_endpoint(text_to_speech);
    result.add_endpoint(speech_enhancement);
    result.add_endpoint(audio_upload);
    result.add_endpoint(audio_process);
    result.add_endpoint(sentiment);
    result.add_endpoint(emotion);

    // Update stats
    result.stats.urls_processed = 4; // Root + 3 main categories
    result.stats.successful_requests = 4;
    result.stats.max_depth_reached = 2;
    result.stats.total_time_ms = 1500;

    result.complete();
    result
}

fn analyze_tree_structure(json: &serde_json::Value) {
    println!("🔍 Structure Analysis:");

    if let Some(api_tree) = json.get("api_tree") {
        if let Some(root) = api_tree.get("http://qx-022160:8080/api/v1") {
            // Check root children
            if let Some(children) = root.get("children") {
                if let Some(children_array) = children.as_array() {
                    println!("  ✅ Root has {} direct children", children_array.len());

                    // Analyze each child
                    for (i, child) in children_array.iter().enumerate() {
                        if let Some(href) = child.get("href") {
                            if let Some(href_str) = href.as_str() {
                                let child_name = href_str.split('/').last().unwrap_or("unknown");

                                if let Some(child_children) = child.get("children") {
                                    if let Some(child_children_array) = child_children.as_array() {
                                        println!(
                                            "    {}. {} has {} children",
                                            i + 1,
                                            child_name,
                                            child_children_array.len()
                                        );
                                    } else {
                                        println!("    {}. {} has no children", i + 1, child_name);
                                    }
                                } else {
                                    println!("    {}. {} has no children", i + 1, child_name);
                                }
                            }
                        }
                    }
                } else {
                    println!("  ❌ Children is not an array");
                }
            } else {
                println!("  ❌ Root has no children");
            }

            // Check for duplicate children arrays
            let children_count = count_children_objects(root);
            if children_count == 1 {
                println!("  ✅ Exactly one 'children' object found");
            } else {
                println!(
                    "  ❌ Found {} 'children' objects (should be 1)",
                    children_count
                );
            }
        } else {
            println!("  ❌ Root node not found in api_tree");
        }
    } else {
        println!("  ❌ api_tree not found in JSON");
    }
    println!();
}

fn count_children_objects(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(obj) => {
            let mut count = 0;
            for (key, val) in obj {
                if key == "children" {
                    count += 1;
                }
                count += count_children_objects(val);
            }
            count
        }
        serde_json::Value::Array(arr) => arr.iter().map(count_children_objects).sum(),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_structure_creation() {
        let result = create_clean_test_structure();

        assert_eq!(result.endpoints.len(), 9);
        assert_eq!(result.start_url, "http://qx-022160:8080/api/v1");

        // Check that we have the right depth distribution
        let depth1_count = result.endpoints_at_depth(1).len();
        let depth2_count = result.endpoints_at_depth(2).len();

        assert_eq!(depth1_count, 3); // generation, audio, analyser
        assert_eq!(depth2_count, 6); // 2 under each of the 3 categories
    }

    #[test]
    fn test_parent_child_relationships() {
        let result = create_clean_test_structure();

        // Check that generation has 2 children
        let generation_children = result
            .url_mappings
            .get("http://qx-022160:8080/api/v1/generation");
        assert!(generation_children.is_some());
        assert_eq!(generation_children.unwrap().len(), 2);

        // Check that audio has 2 children
        let audio_children = result
            .url_mappings
            .get("http://qx-022160:8080/api/v1/audio");
        assert!(audio_children.is_some());
        assert_eq!(audio_children.unwrap().len(), 2);

        // Check that analyser has 2 children
        let analyser_children = result
            .url_mappings
            .get("http://qx-022160:8080/api/v1/analyser");
        assert!(analyser_children.is_some());
        assert_eq!(analyser_children.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_tree_serialization_structure() {
        let result = create_clean_test_structure();

        let config = OutputConfig {
            format: OutputFormat::Tree,
            include_stats: false,
            include_config: false,
            hierarchical: false,
        };

        let json = serialize_result(&result, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify structure
        assert!(parsed.get("api_tree").is_some());
        assert!(parsed.get("start_url").is_some());

        // Check root node structure
        let root = parsed["api_tree"]["http://qx-022160:8080/api/v1"]
            .as_object()
            .unwrap();
        assert!(root.contains_key("children"));
        assert!(root.contains_key("href"));
        assert!(root.contains_key("rel"));

        // Verify exactly one children array
        let children_count =
            count_children_objects(&parsed["api_tree"]["http://qx-022160:8080/api/v1"]);
        assert_eq!(children_count, 4); // Root + 3 children should each have exactly 1 children array
    }
}
