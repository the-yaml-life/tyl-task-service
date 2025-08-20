//! Infrastructure adapters for external systems
//!
//! This module contains adapters for external systems like databases, HTTP clients,
//! message queues, and other infrastructure concerns.

pub mod database;
pub mod http_client;

// Re-export commonly used adapters
pub use database::*;
pub use http_client::*;