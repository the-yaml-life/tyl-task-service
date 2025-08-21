//! Domain module for task management system
//!
//! This module contains the core business logic and domain models for the task management system.
//! It follows hexagonal architecture principles with clear separation between domain and infrastructure.
//! The system uses a graph-based approach through tyl-graph-port and tyl-falkordb-adapter.

pub mod models;
pub mod services;
pub mod queries;

// Re-export commonly used types
pub use models::*;
pub use services::*;
pub use queries::*;