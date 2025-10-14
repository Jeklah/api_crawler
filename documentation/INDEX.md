# Documentation Index

This directory contains all comprehensive documentation for the API Crawler project.

## 📋 Documentation Overview

### Core Documentation
- **[README.md](./README.md)** - Main documentation with installation, usage, and features
- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Technical architecture and design decisions

### Technical Deep-Dives
- **[TREE_ORDERING_SOLUTION.md](./TREE_ORDERING_SOLUTION.md)** - Complete solution for parent-first tree structure
- **[ROOT_CAUSE_ANALYSIS.md](./ROOT_CAUSE_ANALYSIS.md)** - Technical analysis of JSON key ordering issues

### Format Documentation
- **[FORMAT_COMPARISON.md](./FORMAT_COMPARISON.md)** - Before/after comparison of tree output formats
- **[OUTPUT_COMPARISON.md](./OUTPUT_COMPARISON.md)** - Detailed examples of all output formats

## 🚀 Getting Started

1. Start with **[README.md](./README.md)** for installation and basic usage
2. Review **[ARCHITECTURE.md](./ARCHITECTURE.md)** to understand the system design
3. Check **[OUTPUT_COMPARISON.md](./OUTPUT_COMPARISON.md)** for output format examples

## 🔧 Technical Implementation

### Tree Structure Improvements
The API crawler underwent significant improvements to fix tree output formatting issues:

- **Problem**: Children appeared before parent information in JSON output
- **Root Cause**: `serde_json::Map` uses alphabetical key ordering (`BTreeMap`)
- **Solution**: Strategic field renaming (`"endpoint"` → `"api"`) to ensure proper ordering
- **Result**: Clean parent-first tree structure

### Key Documents for Understanding the Fix
1. **[TREE_ORDERING_SOLUTION.md](./TREE_ORDERING_SOLUTION.md)** - Complete solution overview
2. **[ROOT_CAUSE_ANALYSIS.md](./ROOT_CAUSE_ANALYSIS.md)** - Detailed technical investigation
3. **[FORMAT_COMPARISON.md](./FORMAT_COMPARISON.md)** - Before/after structural comparison

## 📊 Output Formats

The API crawler supports multiple output formats:

- **Tree**: Parent-first hierarchical structure with `"api"` and `"children"` fields
- **Hierarchical**: Endpoints nested under parent URLs
- **Pretty**: Formatted JSON with all endpoint details
- **Compact**: Minimized JSON output

See **[OUTPUT_COMPARISON.md](./OUTPUT_COMPARISON.md)** for detailed examples of each format.

## 🏗️ Architecture

The system is built with:
- **Rust** for performance and safety
- **Async/await** for concurrent request handling
- **IndexMap** for insertion-order preservation during tree construction
- **Strategic field naming** to work with JSON alphabetical ordering

## 📈 Development Timeline

### Major Milestones
1. **Initial Implementation** - Basic API crawling functionality
2. **Format Issues Identified** - Tree output showing children before parents
3. **Root Cause Analysis** - Discovered JSON key ordering problem
4. **Solution Implementation** - Field renaming to ensure proper ordering
5. **Documentation Complete** - Comprehensive technical documentation

## 🔗 Related Files

### Example Outputs
- `tree_v2.json` - Example of corrected tree output format
- `api_v1_2022-6_final.json` - Complex API structure example

### Code Structure
- `src/output.rs` - Output formatting and tree building logic
- `src/types.rs` - Core data structures and endpoint definitions
- `src/crawler.rs` - Main crawling engine implementation

## 📝 Notes for Future Development

1. **JSON Ordering**: Remember that `serde_json::Map` uses alphabetical ordering
2. **Field Naming**: When order matters, choose field names strategically
3. **IndexMap Usage**: Use for construction, convert to `serde_json::Map` only for serialization
4. **Testing**: Always verify JSON structure order in output files

## 🤝 Contributing

When adding new documentation:
1. Place all `.md` files in this `documentation/` directory
2. Update this index file with links to new documentation
3. Follow the established naming conventions
4. Include both technical details and practical examples

---

*Last updated: October 2024*