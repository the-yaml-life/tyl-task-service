//! Infrastructure adapters for external systems
//!
//! This module contains adapters for external systems like databases, HTTP clients,
//! message queues, and other infrastructure concerns. The graph_repository provides
//! graph database integration using tyl-graph-port and tyl-falkordb-adapter.

pub mod database;
pub mod graph_repository;
pub mod http_client;

// Re-export commonly used adapters
pub use database::*;
pub use graph_repository::*;
pub use http_client::*;