//! Error types for the API crawler

use thiserror::Error;

/// Main error type for the API crawler
#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("URL parsing failed: {0}")]
    Url(#[from] url::ParseError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid response format: {message}")]
    InvalidResponse { message: String },

    #[error("Maximum depth reached: {depth}")]
    MaxDepthReached { depth: usize },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Timeout occurred")]
    Timeout,

    #[error("Configuration error: {message}")]
    Config { message: String },
}

/// Convenience type alias for Results with CrawlerError
pub type Result<T> = std::result::Result<T, CrawlerError>;

impl CrawlerError {
    /// Create a new InvalidResponse error
    pub fn invalid_response(message: impl Into<String>) -> Self {
        Self::InvalidResponse {
            message: message.into(),
        }
    }

    /// Create a new Config error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }
}
