//! HTTP request handlers for the microservice
//!
//! This module contains all HTTP request handlers organized by functionality.
//! Task-specific handlers provide REST API endpoints for task management.

pub mod health;
pub mod api;
pub mod tasks;

// Re-export commonly used handlers
pub use health::*;
pub use api::*;
pub use tasks::*;