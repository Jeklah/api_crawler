# Changelog

All notable changes to the API Crawler project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- **Major Duplication Bug Fix**: Eliminated all forms of data duplication in hierarchical output format
  - **Metadata Duplication**: Fixed duplicate storage of endpoint properties (`rel`, `method`, `type`, `title`) in both direct fields and metadata objects
  - **Endpoint Duplication**: Fixed complete endpoint entries appearing multiple times due to overlapping extraction strategies
  - **File Size Impact**: 57% reduction in output file size (6.4MB → 2.8MB)
  - **Performance Impact**: Significantly reduced network transfer times and storage requirements
  - **Implementation**: Used idiomatic Rust practices including `HashSet` deduplication, `matches!` macro for field filtering, and custom `Hash` implementation for `EndpointKey`
  - **Testing**: Added comprehensive tests (`test_no_metadata_duplication`, `test_endpoint_deduplication`) to prevent regressions

### Technical Details
- Enhanced `extract_from_link_data()` and `extract_from_object()` functions in `src/crawler.rs`
- Added `EndpointKey` struct with custom `Hash` implementation for efficient deduplication
- Improved extraction strategy coordination to prevent overlapping processing
- Used `matches!` macro for clean field exclusion patterns
- Added proper handling of known endpoint fields (`method`, `type`, `title`) when present in link objects

### Documentation
- Added comprehensive documentation in `documentation/DUPLICATION_BUG_FIX.md`
- Updated documentation index to include duplication fix information
- Documented root cause analysis, implementation details, and performance improvements

## [Previous Versions]

### Tree Ordering Fix
- **Problem**: Children appeared before parent information in JSON output
- **Root Cause**: `serde_json::Map` uses alphabetical key ordering (`BTreeMap`)
- **Solution**: Strategic field renaming (`"endpoint"` → `"api"`) to ensure proper ordering
- **Result**: Clean parent-first tree structure

### Initial Implementation
- Basic API crawling functionality with multiple output formats
- Support for hierarchical, tree, pretty, and compact JSON output
- Async/await concurrent request handling
- Configurable crawling parameters (depth, concurrency, timeouts)

---

**Note**: This changelog follows semantic versioning principles. Breaking changes will increment the major version, new features will increment the minor version, and bug fixes will increment the patch version.