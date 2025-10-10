//! API Crawler CLI application
//!
//! A command-line tool for crawling REST APIs and mapping their endpoint structure.

use api_crawler::output::{
    OutputConfig, OutputFormat, print_endpoints_detailed, print_hierarchical_summary,
    print_summary, save_results_to_file,
};
use api_crawler::prelude::*;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::process;
use tracing::{Level, error, info};
use tracing_subscriber;

#[derive(Parser)]
#[command(
    name = "api_crawler",
    about = "A tool for crawling REST APIs and mapping their endpoint structure",
    version = "1.0.0"
)]
struct Args {
    /// The starting URL to crawl
    #[arg(help = "Starting URL for the API crawl")]
    url: String,

    /// Output file path (defaults to stdout summary if not provided)
    #[arg(short, long, help = "Output file path for JSON results")]
    output: Option<PathBuf>,

    /// Maximum crawling depth (0 = unlimited)
    #[arg(short, long, default_value = "10", help = "Maximum crawling depth")]
    max_depth: usize,

    /// Maximum number of concurrent requests
    #[arg(
        short,
        long,
        default_value = "10",
        help = "Maximum concurrent requests"
    )]
    concurrency: usize,

    /// Request timeout in seconds
    #[arg(short, long, default_value = "30", help = "Request timeout in seconds")]
    timeout: u64,

    /// Maximum number of URLs to crawl (0 = unlimited)
    #[arg(long, default_value = "1000", help = "Maximum number of URLs to crawl")]
    max_urls: usize,

    /// Delay between requests in milliseconds
    #[arg(
        short,
        long,
        default_value = "100",
        help = "Delay between requests (ms)"
    )]
    delay: u64,

    /// Custom User-Agent string
    #[arg(long, default_value = "API-Crawler/1.0", help = "User-Agent string")]
    user_agent: String,

    /// Output format
    #[arg(long, value_enum, default_value = "pretty", help = "Output format")]
    format: OutputFormatArg,

    /// Use hierarchical output structure
    #[arg(long, help = "Structure endpoints under their parent URLs")]
    hierarchical: bool,

    /// Allowed domains (can be specified multiple times)
    #[arg(long, help = "Restrict crawling to these domains")]
    allowed_domain: Vec<String>,

    /// Custom headers in key:value format
    #[arg(long, help = "Custom headers (format: key:value)")]
    header: Vec<String>,

    /// Verbose logging
    #[arg(short, long, help = "Enable verbose logging")]
    verbose: bool,

    /// Show detailed endpoint information
    #[arg(long, help = "Show detailed endpoint information")]
    detailed: bool,

    /// Maximum number of endpoints to show in detailed view
    #[arg(long, default_value = "50", help = "Max endpoints in detailed view")]
    max_show: usize,

    /// Don't follow redirects
    #[arg(long, help = "Don't follow HTTP redirects")]
    no_redirects: bool,
}

#[derive(ValueEnum, Clone)]
enum OutputFormatArg {
    /// Pretty-printed JSON
    Pretty,
    /// Compact JSON
    Compact,
    /// Hierarchical structure with endpoints nested under parent URLs
    Hierarchical,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Pretty => OutputFormat::PrettyJson,
            OutputFormatArg::Compact => OutputFormat::CompactJson,
            OutputFormatArg::Hierarchical => OutputFormat::Hierarchical,
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Validate URL
    if let Err(e) = url::Url::parse(&args.url) {
        error!("Invalid URL '{}': {}", args.url, e);
        process::exit(1);
    }

    // Parse custom headers
    let mut headers = std::collections::HashMap::new();
    for header_str in &args.header {
        if let Some((key, value)) = header_str.split_once(':') {
            headers.insert(key.trim().to_string(), value.trim().to_string());
        } else {
            error!(
                "Invalid header format '{}'. Expected 'key:value'",
                header_str
            );
            process::exit(1);
        }
    }

    // Build crawler configuration
    let mut config = CrawlerConfig::new()
        .max_depth(args.max_depth)
        .max_concurrent_requests(args.concurrency)
        .timeout_seconds(args.timeout);

    config.max_urls = args.max_urls;
    config.delay_ms = args.delay;
    config.user_agent = args.user_agent;
    config.headers = headers;
    config.follow_redirects = !args.no_redirects;

    for domain in args.allowed_domain {
        config = config.allow_domain(domain);
    }

    // Create crawler
    let mut crawler = match ApiCrawler::new(config) {
        Ok(crawler) => crawler,
        Err(e) => {
            error!("Failed to create crawler: {}", e);
            process::exit(1);
        }
    };

    info!("Starting API crawl from: {}", args.url);

    // Start crawling
    let result = match crawler.crawl(&args.url).await {
        Ok(result) => result,
        Err(e) => {
            error!("Crawling failed: {}", e);
            process::exit(1);
        }
    };

    // Output results
    if let Some(output_path) = args.output {
        let output_config = OutputConfig {
            format: args.format.into(),
            include_stats: true,
            include_config: true,
            hierarchical: args.hierarchical,
        };

        if let Err(e) = save_results_to_file(&result, &output_path, Some(output_config)) {
            error!("Failed to save results: {}", e);
            process::exit(1);
        }

        info!("Results saved to: {}", output_path.display());
    }

    // Always print summary to stdout
    print_summary(&result);

    // Print detailed information if requested
    if args.detailed {
        print_endpoints_detailed(&result, Some(args.max_show));
    }

    // Print hierarchical structure if using hierarchical format
    if args.hierarchical {
        print_hierarchical_summary(&result);
    }

    // Exit with appropriate code
    let exit_code = if result.stats.failed_requests == 0 {
        0
    } else {
        2
    };
    process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_conversion() {
        let pretty = OutputFormatArg::Pretty;
        let compact = OutputFormatArg::Compact;
        let hierarchical = OutputFormatArg::Hierarchical;

        matches!(OutputFormat::from(pretty), OutputFormat::PrettyJson);
        matches!(OutputFormat::from(compact), OutputFormat::CompactJson);
        matches!(OutputFormat::from(hierarchical), OutputFormat::Hierarchical);
    }

    #[test]
    fn test_header_parsing() {
        let header_str = "Authorization: Bearer token123";
        if let Some((key, value)) = header_str.split_once(':') {
            assert_eq!(key.trim(), "Authorization");
            assert_eq!(value.trim(), "Bearer token123");
        } else {
            panic!("Failed to parse header");
        }
    }
}
