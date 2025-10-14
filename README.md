# API Crawler

A powerful Rust-based tool for crawling REST APIs and mapping their complete endpoint structure.

## 📚 Documentation

All comprehensive documentation is located in the [`documentation/`](./documentation/) directory:

- **[Main Documentation](./documentation/README.md)** - Complete usage guide, installation, and features
- **[Architecture](./documentation/ARCHITECTURE.md)** - Technical architecture and design decisions
- **[Tree Ordering Solution](./documentation/TREE_ORDERING_SOLUTION.md)** - Details on the parent-first tree structure fix
- **[Format Comparison](./documentation/FORMAT_COMPARISON.md)** - Before/after comparison of output formats
- **[Output Examples](./documentation/OUTPUT_COMPARISON.md)** - Detailed output format examples
- **[Root Cause Analysis](./documentation/ROOT_CAUSE_ANALYSIS.md)** - Technical deep-dive into JSON ordering issues

## Quick Start

```bash
# Install from source
git clone <repository>
cd api_crawler
cargo build --release

# Basic usage
./target/release/api_crawler http://your-api.com/api --format tree

# With options
./target/release/api_crawler http://your-api.com/api \
    --format tree \
    --max-depth 5 \
    --output api-structure.json
```

## Key Features

- 🕷️ **Recursive API Discovery** - Automatically follows JSON links
- 🌳 **Parent-First Tree Structure** - Clean, intuitive hierarchy
- 🚀 **High Performance** - Async with configurable concurrency  
- 📊 **Multiple Output Formats** - Tree, hierarchical, pretty, compact
- 🛡️ **Robust Error Handling** - Graceful failure recovery
- ⚙️ **Highly Configurable** - Extensive customization options

## Contributing

Please see the [documentation directory](./documentation/) for detailed technical information and architecture documentation.

## License

[Add your license here]