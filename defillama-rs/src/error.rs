//! Error types for the DefiLlama API client

use thiserror::Error;

/// Error type for the DefiLlama API client
#[derive(Error, Debug)]
pub enum DefillamaError {
    /// Error returned by reqwest
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    /// URL parsing error
    #[error("URL parsing error: {0}")]
    UrlParseError(#[from] url::ParseError),

    /// Error returned when the API returns an error message
    #[error("API error: {0}")]
    ApiError(String),

    /// Error when parsing the API response
    #[error("Failed to parse API response: {0}")]
    ParseError(String),

    /// Error when a required field is missing in the response
    #[error("Missing field in API response: {0}")]
    MissingField(String),

    /// Any other error
    #[error("Other error: {0}")]
    Other(String),
}
