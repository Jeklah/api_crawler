# API Crawler

A powerful Rust-based tool for crawling REST APIs and mapping their complete endpoint structure. This crawler automatically discovers API endpoints by following links in JSON responses, building a comprehensive map of your API's structure.

## Features

- 🕷️ **Recursive Crawling**: Automatically discovers API endpoints by following JSON links
- 🚀 **High Performance**: Async/await with configurable concurrency limits
- 🔗 **Multiple Link Formats**: Supports HAL (Hypertext Application Language), JSON API, and custom link formats
- 🎯 **Smart Filtering**: Excludes "self" relations to prevent infinite loops
- 📊 **Comprehensive Output**: Clean JSON mapping with statistics and metadata (null fields omitted)
- 🛡️ **Robust Error Handling**: Graceful handling of failures with detailed error reporting
- ⚙️ **Highly Configurable**: Customizable depth, concurrency, timeouts, and more
- 🌐 **Domain Restrictions**: Optionally limit crawling to specific domains
- 📈 **Progress Tracking**: Real-time statistics and progress information
- 🚫 **Zero Duplication**: Advanced deduplication eliminates all redundant data (57% file size reduction!)

## Installation

### From Source

```bash
git clone https://github.com/your-username/api_crawler.git
cd api_crawler
cargo build --release
```

The binary will be available at `target/release/api_crawler`.

### Using Cargo

```bash
cargo install --path .
```

## Quick Start

### Basic Usage

Crawl an API and save results to a JSON file:

```bash
./api_crawler https://api.example.com -o results.json
```

### View Summary Only

Crawl an API and display a summary without saving:

```bash
./api_crawler https://api.example.com
```

### Advanced Usage

```bash
./api_crawler https://api.example.com \
  --output results.json \
  --max-depth 5 \
  --concurrency 20 \
  --timeout 60 \
  --delay 200 \
  --allowed-domain api.example.com \
  --allowed-domain cdn.example.com \
  --header "Authorization: Bearer your-token" \
  --header "Accept: application/hal+json" \
  --verbose \
  --detailed
```

## Command Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--output` | `-o` | Output file path for JSON results | None (stdout summary) |
| `--max-depth` | `-m` | Maximum crawling depth (0 = unlimited) | 10 |
| `--concurrency` | `-c` | Maximum concurrent requests | 10 |
| `--timeout` | `-t` | Request timeout in seconds | 30 |
| `--max-urls` | | Maximum number of URLs to crawl | 1000 |
| `--delay` | `-d` | Delay between requests (ms) | 100 |
| `--user-agent` | | Custom User-Agent string | API-Crawler/1.0 |
| `--format` | | Output format (pretty/compact/hierarchical/tree) | pretty |
| `--hierarchical` | | Structure endpoints under parent URLs | false |
| `--allowed-domain` | | Restrict crawling to these domains | None |
| `--header` | | Custom headers (key:value format) | None |
| `--verbose` | `-v` | Enable verbose logging | false |
| `--detailed` | | Show detailed endpoint information | false |
| `--max-show` | | Max endpoints in detailed view | 50 |
| `--no-redirects` | | Don't follow HTTP redirects | false |

## Supported Link Formats

### HAL (Hypertext Application Language)

```json
{
  "_links": {
    "self": {"href": "/api/users/1"},
    "next": {"href": "/api/users/2"},
    "items": [
      {"href": "/api/users/1/posts"},
      {"href": "/api/users/1/comments"}
    ]
  }
}
```

### JSON API Links

```json
{
  "links": {
    "self": "/api/articles/1",
    "next": "/api/articles/2",
    "related": "/api/articles/1/comments"
  }
}
```

### Custom Link Objects

```json
{
  "data": [
    {
      "href": "/api/endpoint1",
      "rel": "related",
      "method": "GET",
      "type": "application/json"
    }
  ]
}
```

### Direct href Properties

```json
{
  "user_url": "/api/users/1",
  "profile_link": "/api/users/1/profile",
  "avatar_uri": "/api/users/1/avatar"
}
```

## Output Formats

The crawler supports multiple output formats to suit different use cases. All formats automatically omit null/empty fields for cleaner, more concise output.

**Available Formats:**
- **Standard** (`--format pretty`) - Comprehensive flat structure
- **Compact** (`--format compact`) - Minified JSON output  
- **Hierarchical** (`--format hierarchical`) - Nested parent-child structure
- **Tree** (`--format tree`) - Organized tree with inline children *(NEW!)*

### Standard Format (Default)

The default format provides a comprehensive flat structure:

```json
{
  "start_url": "https://api.example.com",
  "endpoints": [
    {
      "href": "https://api.example.com/users",
      "rel": "users",
      "type": "application/json",
      "depth": 1,
      "parent_url": "https://api.example.com",
      "metadata": {
        "discovered_at": "2024-01-15T10:30:00Z"
      }
    }
  ],
  "url_mappings": {
    "https://api.example.com": [...]
  },
  "stats": {
    "urls_processed": 25,
    "successful_requests": 23,
    "failed_requests": 2,
    "urls_skipped": 5,
    "max_depth_reached": 3,
    "total_time_ms": 2500
  },
  "started_at": "2024-01-15T10:29:30Z",
  "completed_at": "2024-01-15T10:30:00Z",
  "config_snapshot": "..."
}
```

### Hierarchical Format

Use `--hierarchical` or `--format hierarchical` to structure endpoints under their parent URLs:

```bash
./api_crawler https://api.example.com --hierarchical -o results.json
```

**Hierarchical Output Structure:**
```json
{
  "start_url": "https://api.example.com",
  "endpoint_hierarchy": {
    "https://api.example.com": [
      {
        "href": "https://api.example.com/users",
        "rel": "users",
        "depth": 1
      },
      {
        "href": "https://api.example.com/posts", 
        "rel": "posts",
        "depth": 1
      }
    ],
    "https://api.example.com/users": [
      {
        "href": "https://api.example.com/users/1",
        "rel": "user",
        "depth": 2
      },
      {
        "href": "https://api.example.com/users/1/posts",
        "rel": "user-posts", 
        "depth": 2
      }
    ]
  },
  "summary": {
    "total_endpoints": 4,
    "unique_parents": 2,
    "discovered_domains": 1
  },
  "stats": { ... },
  "started_at": "2024-01-15T10:29:30Z",
  "completed_at": "2024-01-15T10:30:00Z"
}
```

**Benefits of Hierarchical Format:**
- 🌳 **Clear parent-child relationships** - Easy to see which endpoints were discovered from where
- 📊 **Better visualization** - Natural tree structure for API exploration
- 🔍 **Simplified navigation** - Intuitive browsing of discovered endpoints
- 📚 **Documentation friendly** - Perfect for generating API documentation
- 🧹 **Clean output** - Automatically omits null fields (`method`, `type`, `title`) and empty collections

### Tree Format (NEW!)

Use `--format tree` for the most organized and compact structure:

```bash
./api_crawler https://api.example.com --format tree -o results.json
```

**Tree Output Structure:**
```json
{
  "start_url": "https://api.example.com",
  "api_tree": {
    "https://api.example.com": {
      "href": "https://api.example.com",
      "rel": "root",
      "depth": 0,
      "children": [
        {
          "href": "https://api.example.com/users",
          "rel": "users",
          "depth": 1,
          "children": [
            {
              "href": "https://api.example.com/users/1",
              "rel": "user",
              "depth": 2,
              "children": [
                {
                  "href": "https://api.example.com/users/1/posts",
                  "rel": "user-posts",
                  "depth": 3
                }
              ]
            }
          ]
        }
      ]
    }
  },
  "summary": {
    "total_endpoints": 3,
    "max_depth": 3,
    "discovered_domains": 1
  }
}
```

**Benefits of Tree Format:**
- 🌳 **Ultimate organization** - Each endpoint contains all its children inline
- 📦 **Maximum compactness** - Eliminates ALL redundant references
- 🔍 **Easy navigation** - Natural tree structure for programmatic access
- 📉 **Smallest file size** - Most efficient format (50-70% size reduction)
- 🎯 **Perfect for docs** - Ideal for generating API documentation trees

## Library Usage

You can also use the API crawler as a Rust library:

```toml
[dependencies]
api_crawler = { path = "." }
tokio = { version = "1.0", features = ["full"] }
```

```rust
use api_crawler::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config = CrawlerConfig::new()
        .max_depth(5)
        .max_concurrent_requests(10)
        .timeout_seconds(30);

    let mut crawler = ApiCrawler::new(config)?;
    let result = crawler.crawl("https://api.example.com").await?;

    println!("Found {} endpoints", result.endpoints.len());
    println!("Summary: {}", result.summary());

    Ok(())
}
```

## Configuration Examples

### Output Format Examples

**Tree format (recommended):**
```bash
./api_crawler https://api.example.com --format tree -o api_tree.json
```

**Hierarchical format:**
```bash
./api_crawler https://api.example.com --hierarchical -o api_hierarchy.json
```

**Comparison of all formats:**
```bash
# Standard flat format
./api_crawler https://api.example.com -o standard.json

# Hierarchical nested format  
./api_crawler https://api.example.com --hierarchical -o hierarchical.json

# Tree format (most organized)
./api_crawler https://api.example.com --format tree -o tree.json

# Compact tree format
./api_crawler https://api.example.com --format tree | jq -c . > compact_tree.json
```

### High-Speed Crawling

```bash
./api_crawler https://api.example.com \
  --concurrency 50 \
  --delay 0 \
  --timeout 10 \
  --max-urls 10000
```

### Conservative Crawling

```bash
./api_crawler https://api.example.com \
  --concurrency 2 \
  --delay 1000 \
  --timeout 60 \
  --max-depth 3
```

### Authentication Example

```bash
./api_crawler https://api.example.com \
  --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..." \
  --header "Accept: application/hal+json" \
  --header "User-Agent: MyApp/1.0"
```

## Use Cases

### API Documentation
- Automatically discover all available endpoints
- Generate comprehensive API maps
- Validate API structure and links

### Testing & QA
- Verify endpoint accessibility
- Check for broken links
- Validate response formats

### Security Auditing
- Discover hidden or undocumented endpoints
- Map attack surfaces
- Check access controls

### API Migration
- Compare API versions
- Identify endpoint changes
- Validate migration completeness

## Best Practices

1. **Start with low concurrency** when crawling unknown APIs
2. **Use domain restrictions** to avoid crawling external resources
3. **Set appropriate timeouts** based on API response times
4. **Include authentication headers** for protected APIs
5. **Monitor rate limits** and adjust delay accordingly
6. **Save results to files** for later analysis
7. **Use verbose mode** for debugging issues
8. **Use tree format** for the most organized and compact output
9. **Use hierarchical format** for nested parent-child relationships

## Common Issues

### Rate Limiting
If you encounter rate limiting, reduce concurrency and increase delay:
```bash
--concurrency 1 --delay 2000
```

### Authentication Errors
Ensure proper authentication headers:
```bash
--header "Authorization: Bearer YOUR_TOKEN"
```

### Timeout Issues
Increase timeout for slow APIs:
```bash
--timeout 120
```

### Memory Usage
Limit the crawl scope for large APIs:
```bash
--max-urls 500 --max-depth 3
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Changelog

### v1.0.0
- Initial release
- HAL and JSON API support
- Concurrent crawling
- Multiple output formats: standard, hierarchical, and tree
- Clean JSON output (null fields automatically omitted)
- Tree format for maximum organization and compactness
- CLI interface
- Library interface