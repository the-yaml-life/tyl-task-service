//! HTTP request handlers for the microservice
//!
//! This module contains all HTTP request handlers organized by functionality.

pub mod health;
pub mod api;

// Re-export commonly used handlers
pub use health::*;
pub use api::*;