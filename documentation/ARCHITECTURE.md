# API Crawler Architecture

## Overview

The API Crawler is a high-performance, async Rust application designed to systematically discover and map REST API endpoints by following JSON links. It starts from a given URL and recursively crawls discovered endpoints, building a comprehensive map of the API structure.

## Project Structure

```
api_crawler/
├── src/
│   ├── lib.rs           # Library entry point and module exports
│   ├── main.rs          # CLI application entry point
│   ├── crawler.rs       # Core crawling logic and HTTP client
│   ├── types.rs         # Data structures and type definitions
│   ├── error.rs         # Error handling and custom error types
│   └── output.rs        # JSON serialization and result formatting
├── examples/
│   ├── simple_test.rs   # Basic library usage example
│   └── hal_api_test.rs  # HAL API crawling demonstration
├── Cargo.toml           # Dependencies and project metadata
└── README.md            # Usage documentation
```

## Core Architecture

### 1. Modular Design (Modern Rust Style)

The project follows modern Rust patterns with a single entry point (`lib.rs`) that exports all modules, avoiding the older `mod.rs` approach.

### 2. Type System

#### Core Types
- **`ApiEndpoint`**: Represents a discovered API endpoint with metadata
- **`CrawlResult`**: Complete crawling results with statistics
- **`CrawlerConfig`**: Configuration for crawling behavior
- **`QueueItem`**: Internal queue management for URLs to process

#### Error Handling
- Uses `thiserror` for ergonomic error definitions
- Custom `CrawlerError` enum covering all failure modes
- Result-based error propagation throughout

### 3. Async Architecture

```rust
ApiCrawler -> reqwest::Client -> HTTP Requests
     ↓
  JSON Parser -> Endpoint Extractor -> Queue Manager
     ↓
  Result Aggregator -> Output Formatter
```

#### Concurrency Control
- `tokio::sync::Semaphore` for limiting concurrent requests
- Configurable concurrency levels (default: 10)
- Built-in rate limiting with configurable delays

## Algorithm Flow

### 1. Initialization Phase
```
1. Parse configuration and validate parameters
2. Create HTTP client with custom headers and timeouts
3. Initialize URL queue with starting URL
4. Set up concurrency control semaphore
```

### 2. Crawling Loop
```
While queue not empty and limits not reached:
  1. Dequeue URL item
  2. Check depth and domain restrictions
  3. Skip if already visited (cycle detection)
  4. Make HTTP request with rate limiting
  5. Parse JSON response
  6. Extract endpoints using multiple strategies
  7. Filter out "self" relations
  8. Add new URLs to queue
  9. Update statistics and results
```

### 3. Endpoint Extraction Strategies

The crawler uses multiple extraction patterns to maximize API discovery:

#### HAL (Hypertext Application Language)
```json
{
  "_links": {
    "self": {"href": "/api/users/1"},
    "next": {"href": "/api/users/2"},
    "posts": {"href": "/api/users/1/posts"}
  }
}
```

#### JSON API Links
```json
{
  "links": {
    "self": "/api/articles/1",
    "next": "/api/articles/2"
  }
}
```

#### Direct href Attributes
```json
{
  "user_url": "/api/users/1",
  "profile_link": "/api/users/1/profile"
}
```

#### Embedded Objects
Recursively searches nested objects and arrays for link patterns.

## Key Features

### 1. Robust Link Discovery
- Multiple link format support (HAL, JSON API, custom)
- Recursive JSON parsing for nested structures
- Pattern matching for URL-like strings
- Metadata extraction and preservation

### 2. Intelligent Crawling
- Automatic cycle detection using visited URL tracking
- Configurable depth limits to prevent runaway crawling
- Domain restrictions for security and focus
- Self-relation filtering to avoid infinite loops

### 3. Performance Optimization
- Async/await throughout for non-blocking I/O
- Configurable concurrent request limits
- HTTP connection pooling via reqwest
- Efficient memory usage with streaming JSON parsing

### 4. Comprehensive Configuration
```rust
CrawlerConfig {
    max_depth: usize,           // Crawling depth limit
    max_concurrent_requests: usize, // Concurrency control
    timeout_seconds: u64,       // Request timeout
    max_urls: usize,            // Total URL limit
    delay_ms: u64,              // Rate limiting delay
    user_agent: String,         // Custom user agent
    headers: HashMap<String, String>, // Custom headers
    follow_redirects: bool,     // Redirect handling
    allowed_domains: HashSet<String>, // Domain whitelist
}
```

### 5. Rich Output Format
- Complete endpoint inventory with metadata
- URL mapping showing parent-child relationships  
- Comprehensive statistics (success/failure rates, timing)
- Error tracking and reporting
- Configurable JSON output (pretty/compact)

## CLI Interface

The command-line interface provides easy access to all crawler functionality:

```bash
api_crawler <URL> [OPTIONS]
  --output/-o <FILE>          # Save results to JSON file
  --max-depth/-m <N>          # Maximum crawling depth
  --concurrency/-c <N>        # Concurrent requests
  --timeout/-t <SECONDS>      # Request timeout
  --delay/-d <MS>             # Delay between requests
  --header <KEY:VALUE>        # Custom HTTP headers
  --allowed-domain <DOMAIN>   # Domain restrictions
  --verbose/-v                # Debug logging
  --detailed                  # Show endpoint details
```

## Library Interface

The crawler can be used programmatically:

```rust
use api_crawler::prelude::*;

let config = CrawlerConfig::new()
    .max_depth(5)
    .max_concurrent_requests(10);

let mut crawler = ApiCrawler::new(config)?;
let result = crawler.crawl("https://api.example.com").await?;

println!("Found {} endpoints", result.endpoints.len());
```

## Dependencies

### Core Dependencies
- **tokio**: Async runtime and utilities
- **reqwest**: HTTP client with async support
- **serde**: Serialization framework
- **serde_json**: JSON parsing and generation
- **url**: URL parsing and validation
- **chrono**: Date/time handling

### Utility Dependencies  
- **thiserror**: Error handling macros
- **tracing**: Structured logging
- **clap**: Command-line argument parsing

## Testing Strategy

### Unit Tests
- Individual component testing (endpoint extraction, configuration)
- Mock JSON response testing
- Error condition validation

### Integration Tests
- Example applications demonstrating real usage
- HAL and JSON API format testing
- Performance benchmarking scenarios

## Security Considerations

### Input Validation
- URL parsing and validation before requests
- Domain whitelist enforcement
- Header validation and sanitization

### Rate Limiting
- Built-in request delays to prevent abuse
- Concurrency limits to avoid overwhelming targets
- Timeout protection against hanging requests

### Error Handling
- Graceful failure handling with detailed error reporting
- No sensitive information leakage in error messages
- Resource cleanup on failure conditions

## Performance Characteristics

### Throughput
- Configurable concurrency (1-100+ concurrent requests)
- HTTP/2 connection reuse via reqwest
- Efficient JSON parsing with serde_json

### Memory Usage
- Streaming JSON parsing for large responses
- Visited URL deduplication with HashSet
- Bounded queue size with configurable limits

### Scalability
- Horizontal scaling through multiple crawler instances
- Vertical scaling with increased concurrency
- Domain partitioning for large API discovery

## Future Enhancements

### Planned Features
- GraphQL endpoint discovery
- OpenAPI/Swagger integration  
- Custom extraction rule configuration
- Distributed crawling coordination
- Real-time progress monitoring
- Export to various formats (CSV, XML, etc.)

### Performance Improvements
- Connection pooling optimization
- Smarter retry logic with backoff
- Caching layer for repeated requests
- Parallel domain processing

## Conclusion

The API Crawler provides a robust, performant solution for REST API discovery and mapping. Its modular architecture, comprehensive configuration options, and multiple output formats make it suitable for various use cases from API documentation to security auditing.

The combination of Rust's performance and safety guarantees with modern async programming patterns results in a tool that can handle large-scale API crawling tasks efficiently and reliably.