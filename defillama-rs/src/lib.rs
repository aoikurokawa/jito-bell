//! A Rust client for the DefiLlama API
//!
//! This crate provides an interface to interact with DefiLlama's API,
//! allowing users to fetch data such as token prices, TVL, and more.

pub mod client;
pub mod error;
pub mod models;

pub use client::DefiLlamaClient;
pub use error::DefillamaError;
