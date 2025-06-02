//! Data models for the DefiLlama API responses

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Response from the /prices/current endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PriceResponse {
    /// Map of coin identifier to price data
    pub coins: HashMap<String, CoinData>,
}

/// Price information for a specific coin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinData {
    /// Current price in USD
    pub price: f64,
    /// Token symbol (e.g., "ETH", "SOL")
    #[serde(default)]
    pub symbol: String,
    /// Timestamp of the price data (Unix timestamp)
    #[serde(default)]
    pub timestamp: u64,
    /// Confidence score (0-1) indicating reliability of price data
    #[serde(default)]
    pub confidence: f64,
    /// Decimals for the token
    #[serde(default)]
    pub decimals: Option<u8>,
}

/// Supported chains in DefiLlama
#[derive(Debug)]
pub enum Chain {
    Ethereum,
    Solana,
    Polygon,
    Avalanche,
    BinanceSmartChain,
    Arbitrum,
    Optimism,
    Fantom,
    // Add more chains as needed
}

impl Chain {
    /// Get the chain identifier as used in the DefiLlama API
    pub fn as_str(&self) -> &'static str {
        match self {
            Chain::Ethereum => "ethereum",
            Chain::Solana => "solana",
            Chain::Polygon => "polygon",
            Chain::Avalanche => "avax",
            Chain::BinanceSmartChain => "bsc",
            Chain::Arbitrum => "arbitrum",
            Chain::Optimism => "optimism",
            Chain::Fantom => "fantom",
        }
    }
}

/// Helper to format a token for the API (chain:address)
#[derive(Debug)]
pub struct Token {
    /// Chain of the token
    pub chain: Chain,
    /// Address of the token (or token identifier)
    pub address: String,
}

impl Token {
    /// Create a new token with the given chain and address
    pub fn new(chain: Chain, address: String) -> Self {
        Self { chain, address }
    }

    /// Format the token for the API as chain:address
    pub fn format(&self) -> String {
        format!("{}:{}", self.chain.as_str(), self.address)
    }

    /// Get the native token for a chain (using the zero address)
    pub fn native(chain: Chain) -> Self {
        Self {
            chain,
            address: "0x0000000000000000000000000000000000000000".to_string(),
        }
    }
}
