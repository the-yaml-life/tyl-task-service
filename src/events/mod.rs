//! Event handling module for TYL microservice
//!
//! This module provides:
//! - Event publishing capabilities using tyl-pubsub-port
//! - Event handler trait and base implementations
//! - Event routing and dispatching
//! - Example events and handlers
//!
//! ## Quick Start
//!
//! ```rust
//! use tyl_microservice::events::{EventService, DomainEventHandler};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct TaskRegistered {
//!     task_id: String,
//!     email: String,
//! }
//!
//! // Create event service
//! let event_service = EventService::new().await?;
//!
//! // Publish event
//! let event = TaskRegistered {
//!     task_id: "123".to_string(),
//!     email: "task@example.com".to_string(),
//! };
//! event_service.publish("task.registered", event).await?;
//! ```

pub mod service;
pub mod handlers;
pub mod examples;

// Re-export commonly used types
pub use service::EventService;
pub use handlers::{DomainEventHandler, EventHandlerResult};

// Re-export tyl-pubsub-port types for convenience
pub use tyl_pubsub_port::{
    EventPublisher, EventSubscriber, EventHandler, MockPubSubAdapter,
    Event, EventId, SubscriptionId, HandlerResult
};