//! API Crawler Library
//!
//! A Rust library for crawling REST APIs and mapping their endpoint structure.

pub mod crawler;
pub mod error;
pub mod output;
pub mod types;

pub use crawler::ApiCrawler;
pub use error::{CrawlerError, Result};
pub use types::{ApiEndpoint, CrawlResult, CrawlerConfig};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{ApiCrawler, CrawlerError, Result, ApiEndpoint, CrawlResult, CrawlerConfig};
}
