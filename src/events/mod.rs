//! Event handling module for TYL task service
//!
//! This module provides:
//! - Event publishing capabilities using tyl-pubsub-port
//! - Task-specific domain events for event-driven communication
//! - Event handler trait and base implementations
//! - Event routing and dispatching
//!
//! ## Quick Start
//!
//! ```rust
//! use tyl_task_service::events::{EventService, TaskCreated};
//! 
//! // Create event service
//! let event_service = EventService::new().await?;
//!
//! // Publish task created event
//! let event = TaskCreated {
//!     task_id: "PROJ1-T001".to_string(),
//!     name: "New Task".to_string(),
//!     context: TaskContext::Work,
//!     priority: TaskPriority::High,
//!     assigned_user_id: Some("user123".to_string()),
//!     project_id: Some("PROJ1".to_string()),
//!     created_at: Utc::now(),
//! };
//! event_service.publish("task.created", event).await?;
//! ```

pub mod service;
pub mod handlers;
pub mod examples;
pub mod task_events;

// Re-export commonly used types
pub use service::EventService;
pub use handlers::{DomainEventHandler, EventHandlerResult};
pub use task_events::*;

// Re-export tyl-pubsub-port types for convenience
pub use tyl_pubsub_port::{
    EventPublisher, EventSubscriber, EventHandler, MockPubSubAdapter,
    Event, EventId, SubscriptionId, HandlerResult
};