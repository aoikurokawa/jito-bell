//! Client implementation for the DefiLlama API

use std::time::Duration;

use reqwest::Client as HttpClient;
use url::Url;

use crate::{
    error::DefillamaError,
    models::{PriceResponse, Token},
};

/// Base URLs for different DefiLlama API endpoints
pub struct ApiEndpoints {
    /// Base URL for the prices API
    pub prices: &'static str,

    /// Base URL for the TVL API
    pub tvl: &'static str,

    /// Base URL for the stablecoins API
    pub stablecoins: &'static str,

    /// Base URL for the yields API
    pub yields: &'static str,
}

impl Default for ApiEndpoints {
    fn default() -> Self {
        Self {
            prices: "https://coins.llama.fi",
            tvl: "https://api.llama.fi",
            stablecoins: "https://stablecoins.llama.fi",
            yields: "https://yields.llama.fi",
        }
    }
}

/// Client for the DefiLlama API
pub struct DefiLlamaClient {
    /// HTTP client for making API requests
    http_client: HttpClient,

    /// API endpoints to use
    endpoints: ApiEndpoints,
}

impl Default for DefiLlamaClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DefiLlamaClient {
    /// Create a new DefiLlama API client with default configuration
    pub fn new() -> Self {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            endpoints: ApiEndpoints::default(),
        }
    }

    /// Create a new DefiLlama API client with a custom HTTP client
    pub fn with_http_client(http_client: HttpClient) -> Self {
        Self {
            http_client,
            endpoints: ApiEndpoints::default(),
        }
    }

    /// Create a new DefiLlama API client with custom API endpoints
    pub fn with_endpoints(endpoints: ApiEndpoints) -> Self {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            endpoints,
        }
    }

    /// Get current price for a single token
    pub async fn get_price(&self, token: &Token) -> Result<PriceResponse, DefillamaError> {
        let url = format!(
            "{}/prices/current/{}",
            self.endpoints.prices,
            token.format()
        );
        let url = Url::parse(&url)?;

        let response = self.http_client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(DefillamaError::ApiError(format!(
                "API returned error status: {}",
                response.status()
            )));
        }

        let price_response = response.json::<PriceResponse>().await?;
        Ok(price_response)
    }

    /// Get current prices for multiple tokens at once
    pub async fn get_prices(&self, tokens: &[Token]) -> Result<PriceResponse, DefillamaError> {
        if tokens.is_empty() {
            return Err(DefillamaError::Other(
                "Token list cannot be empty".to_string(),
            ));
        }

        // Format tokens as comma-separated list
        let token_list = tokens
            .iter()
            .map(|token| token.format())
            .collect::<Vec<_>>()
            .join(",");

        let url = format!("{}/prices/current/{}", self.endpoints.prices, token_list);
        let url = Url::parse(&url)?;

        let response = self.http_client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(DefillamaError::ApiError(format!(
                "API returned error status: {}",
                response.status()
            )));
        }

        let price_response = response.json::<PriceResponse>().await?;
        Ok(price_response)
    }

    // You can add more API methods here for other endpoints
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Chain;

    #[tokio::test]
    async fn test_get_price() {
        let client = DefiLlamaClient::new();
        let token = Token::native(Chain::Ethereum);

        let result = client.get_price(&token).await;
        assert!(result.is_ok(), "Failed to get price: {:?}", result.err());

        let price_response = result.unwrap();
        assert!(!price_response.coins.is_empty(), "No coin data returned");
    }

    #[tokio::test]
    async fn test_get_prices() {
        let client = DefiLlamaClient::new();
        let tokens = vec![Token::native(Chain::Ethereum), Token::native(Chain::Solana)];

        let result = client.get_prices(&tokens).await;
        assert!(result.is_ok(), "Failed to get prices: {:?}", result.err());

        let price_response = result.unwrap();
        assert!(!price_response.coins.is_empty(), "No coin data returned");
        assert!(price_response.coins.len() >= 1, "Not all tokens returned");
    }
}
