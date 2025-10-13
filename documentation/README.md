# API Crawler Documentation

This directory contains comprehensive documentation for the API Crawler project.

## Documentation Files

### Architecture & Design
- [`ARCHITECTURE.md`](ARCHITECTURE.md) - Detailed technical architecture, design patterns, and implementation details
- [`FORMAT_COMPARISON.md`](FORMAT_COMPARISON.md) - Comprehensive comparison of all output formats with examples
- [`OUTPUT_COMPARISON.md`](OUTPUT_COMPARISON.md) - Before/after comparison showing improvements from clean output

## Quick Reference

### Output Formats

| Format | Command Flag | Best For | File Size |
|--------|--------------|----------|-----------|
| **Tree** | `--format tree` | **Documentation & Visualization** | **Smallest readable** |
| Hierarchical | `--hierarchical` | Analysis & Grouping | Medium |
| Standard | `--format pretty` | General Processing | Large |
| Compact | `--format compact` | Data Transmission | Smallest unreadable |

### Common Commands

```bash
# Recommended: Tree format
./api_crawler https://api.example.com --format tree -o results.json

# Analysis: Hierarchical format
./api_crawler https://api.example.com --hierarchical -o analysis.json

# Processing: Standard format
./api_crawler https://api.example.com -o processing.json

# Transmission: Compact format  
./api_crawler https://api.example.com --format compact -o compact.json
```

## Key Features Documentation

### 🌳 Tree Format (Recommended)
The tree format represents the ultimate evolution of API structure visualization:
- **Inline children**: Each endpoint contains all its children directly
- **Zero redundancy**: No duplicate parent-child references
- **50-70% size reduction**: Most compact readable format
- **Perfect organization**: Natural tree structure for documentation

See [`FORMAT_COMPARISON.md`](FORMAT_COMPARISON.md) for detailed examples.

### 🔧 Technical Architecture
The crawler uses modern Rust patterns:
- **Async/await**: Non-blocking I/O throughout
- **Modular design**: Single `lib.rs` entry point (modern style)
- **Type safety**: Comprehensive error handling with `thiserror`
- **Performance**: Configurable concurrency with semaphores

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for implementation details.

### 🧹 Clean Output
All formats automatically omit null/empty fields:
- **Removed fields**: `method: null`, `type: null`, `title: null`
- **Empty collections**: `errors: []`, `metadata: {}`
- **Zero values**: `failed_requests: 0`, `urls_skipped: 0`

See [`OUTPUT_COMPARISON.md`](OUTPUT_COMPARISON.md) for before/after examples.

## Usage Scenarios

### 📚 API Documentation Generation
```bash
# Generate documentation-ready tree structure
./api_crawler https://api.example.com --format tree --max-depth 5 -o api_docs.json

# Include detailed endpoint information
./api_crawler https://api.example.com --format tree --detailed -o detailed_docs.json
```

### 🔍 API Analysis & Testing
```bash
# Analyze API structure by parent-child relationships
./api_crawler https://api.example.com --hierarchical --verbose -o analysis.json

# Test with authentication
./api_crawler https://api.example.com \
  --header "Authorization: Bearer TOKEN" \
  --format tree -o authenticated.json
```

### 🚀 Production Data Processing
```bash
# High-speed crawling with rate limiting
./api_crawler https://api.example.com \
  --concurrency 20 --delay 100 \
  --max-urls 5000 -o production.json

# Conservative crawling for sensitive APIs
./api_crawler https://api.example.com \
  --concurrency 2 --delay 1000 \
  --max-depth 3 -o conservative.json
```

### 🔒 Security Auditing
```bash
# Discover all endpoints with domain restrictions
./api_crawler https://api.example.com \
  --allowed-domain api.example.com \
  --allowed-domain cdn.example.com \
  --format tree -o security_audit.json
```

## Performance Guidelines

### Recommended Settings by API Size

| API Size | Concurrency | Delay | Max URLs | Max Depth |
|----------|-------------|-------|----------|-----------|
| Small (< 50 endpoints) | 5 | 100ms | 100 | 5 |
| Medium (< 500 endpoints) | 10 | 100ms | 1000 | 10 |
| Large (< 5000 endpoints) | 20 | 50ms | 5000 | 15 |
| Very Large (> 5000 endpoints) | 30 | 25ms | 10000 | 20 |

### Rate Limiting Guidelines

```bash
# Conservative (1 req/sec)
--concurrency 1 --delay 1000

# Moderate (5 req/sec)  
--concurrency 5 --delay 200

# Aggressive (20 req/sec)
--concurrency 20 --delay 50
```

## Supported API Formats

### HAL (Hypertext Application Language)
```json
{
  "_links": {
    "self": {"href": "/api/users/1"},
    "next": {"href": "/api/users/2"},
    "posts": {"href": "/api/users/1/posts"}
  }
}
```

### JSON API
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
  "user_url": "/api/users/1",
  "profile_link": "/api/users/1/profile",
  "data": [
    {"href": "/api/endpoint", "rel": "related"}
  ]
}
```

## Error Handling & Troubleshooting

### Common Issues
- **Connection failures**: Check URL and network connectivity
- **Rate limiting**: Reduce concurrency or increase delay
- **Memory issues**: Limit max-urls and max-depth
- **Tree format panics**: Use debug mode for diagnosis

### Debug Commands
```bash
# Full debugging
./api_crawler URL --debug --verbose --format tree

# Fallback to standard format on tree errors
./api_crawler URL --debug --format tree -o output.json
```

## Migration Guide

### From Other Tools
- **Postman collections**: Use tree format for similar hierarchical structure
- **OpenAPI/Swagger**: Standard format for flat processing compatibility
- **Custom scrapers**: Hierarchical format for grouped analysis

### Between Formats
- **Standard → Tree**: Best for documentation and visualization
- **Standard → Hierarchical**: Good for parent-child analysis
- **Any → Compact**: Use for data transmission only

## Contributing to Documentation

When updating documentation:
1. Keep examples practical and tested
2. Update format comparisons when adding features
3. Include performance implications
4. Maintain consistency across files
5. Test all command examples

## Further Reading

- [Main README](../README.md) - Usage examples and installation
- [Test Directory](../test/README.md) - Testing procedures and examples
- [Examples](../examples/) - Code examples and demonstrations

For questions or issues, see the main project README or create an issue on the repository.