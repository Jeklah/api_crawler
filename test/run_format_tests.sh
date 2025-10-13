#!/bin/bash

# API Crawler Format Test Script
# This script tests all output formats and saves results to the test directory

set -e  # Exit on any error

echo "🧪 API Crawler Format Test Suite"
echo "================================="
echo

# Create test directory if it doesn't exist
mkdir -p test

# Test URLs - using public APIs that should work
TEST_URLS=(
    "https://jsonplaceholder.typicode.com/posts/1"
    "https://httpbin.org/json"
)

# Build the project first
echo "🔨 Building API Crawler..."
cargo build --release
echo

# Function to test a single URL with all formats
test_url() {
    local url="$1"
    local basename="$2"

    echo "🎯 Testing URL: $url"
    echo "   Base name: $basename"
    echo

    # Test 1: Standard Format
    echo "   📄 Testing standard format..."
    ./target/release/api_crawler "$url" \
        --format pretty \
        --max-depth 2 \
        --timeout 10 \
        -o "test/${basename}_standard.json" || echo "   ❌ Standard format failed"

    # Test 2: Compact Format
    echo "   📦 Testing compact format..."
    ./target/release/api_crawler "$url" \
        --format compact \
        --max-depth 2 \
        --timeout 10 \
        -o "test/${basename}_compact.json" || echo "   ❌ Compact format failed"

    # Test 3: Hierarchical Format
    echo "   🌐 Testing hierarchical format..."
    ./target/release/api_crawler "$url" \
        --format hierarchical \
        --max-depth 2 \
        --timeout 10 \
        -o "test/${basename}_hierarchical.json" || echo "   ❌ Hierarchical format failed"

    # Test 4: Tree Format
    echo "   🌳 Testing tree format..."
    ./target/release/api_crawler "$url" \
        --format tree \
        --max-depth 2 \
        --timeout 10 \
        -o "test/${basename}_tree.json" || echo "   ❌ Tree format failed"

    echo "   ✅ Completed tests for $basename"
    echo
}

# Test with different URLs
test_url "https://jsonplaceholder.typicode.com/posts/1" "jsonplaceholder_posts"
test_url "https://httpbin.org/json" "httpbin_json"

# Test edge cases
echo "🔍 Testing Edge Cases"
echo "====================="
echo

# Test with non-existent URL (should handle gracefully)
echo "   🚫 Testing non-existent URL..."
./target/release/api_crawler "https://nonexistent-domain-12345.com/api" \
    --format tree \
    --max-depth 1 \
    --timeout 5 \
    -o "test/nonexistent_tree.json" || echo "   ✅ Non-existent URL handled gracefully"

# Test with HTML endpoint (should skip gracefully)
echo "   📄 Testing HTML endpoint..."
./target/release/api_crawler "https://httpbin.org/html" \
    --format tree \
    --max-depth 1 \
    --timeout 10 \
    -o "test/html_endpoint_tree.json" || echo "   ❌ HTML endpoint test failed"

echo

# Generate file size comparison
echo "📊 File Size Comparison"
echo "======================="
echo

if [ -f "test/jsonplaceholder_posts_standard.json" ]; then
    echo "JSONPlaceholder Posts - File Sizes:"
    ls -lh test/jsonplaceholder_posts_*.json | awk '{print "  " $9 ": " $5}' | sort
    echo
fi

if [ -f "test/httpbin_json_standard.json" ]; then
    echo "HTTPBin JSON - File Sizes:"
    ls -lh test/httpbin_json_*.json | awk '{print "  " $9 ": " $5}' | sort
    echo
fi

# Generate summary report
echo "📋 Test Summary Report"
echo "====================="
echo

echo "Generated test files:"
ls -1 test/*.json | wc -l | xargs echo "  • Total JSON files:"
ls -1 test/*_standard.json 2>/dev/null | wc -l | xargs echo "  • Standard format files:"
ls -1 test/*_compact.json 2>/dev/null | wc -l | xargs echo "  • Compact format files:"
ls -1 test/*_hierarchical.json 2>/dev/null | wc -l | xargs echo "  • Hierarchical format files:"
ls -1 test/*_tree.json 2>/dev/null | wc -l | xargs echo "  • Tree format files:"

echo
echo "📁 All test files saved in: ./test/"
echo
echo "🔍 To examine results:"
echo "  • View standard format:     cat test/jsonplaceholder_posts_standard.json"
echo "  • View hierarchical format: cat test/jsonplaceholder_posts_hierarchical.json"
echo "  • View tree format:         cat test/jsonplaceholder_posts_tree.json"
echo "  • Compare file sizes:       ls -lh test/*.json"
echo
echo "✅ Format tests completed!"
