//! Domain event handlers and utilities

use async_trait::async_trait;
use tyl_pubsub_port::{Event, EventHandler, HandlerResult, HandlerError, RetryPolicy};

/// Convenience type alias for event handler results
pub type EventHandlerResult = HandlerResult;

/// Simple domain event handler trait for business logic
/// 
/// This trait provides a clean interface for handling domain events
/// without dealing with event metadata or complex error handling.
#[async_trait]
pub trait DomainEventHandler<T>: Send + Sync {
    /// Handle a domain event
    /// 
    /// # Arguments
    /// * `event` - The event payload to process
    /// 
    /// # Returns
    /// Result indicating success or failure
    async fn handle_domain_event(&self, event: T) -> HandlerResult;
}

/// Adapter that implements EventHandler for any DomainEventHandler
/// 
/// This makes it easy to use simple domain handlers with the event system.
pub struct DomainEventHandlerAdapter<T, H> {
    handler: H,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, H> DomainEventHandlerAdapter<T, H>
where
    H: DomainEventHandler<T>,
{
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T, H> EventHandler<T> for DomainEventHandlerAdapter<T, H>
where
    T: Send + Sync + 'static,
    H: DomainEventHandler<T> + 'static,
{
    async fn handle(&self, event: Event<T>) -> HandlerResult {
        // Simply extract the payload and pass it to the domain handler
        self.handler.handle_domain_event(event.payload).await
    }
}

/// Simple logging event handler for testing/development
pub struct LoggingEventHandler {
    name: String,
}

impl LoggingEventHandler {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
}

#[async_trait]
impl<T> DomainEventHandler<T> for LoggingEventHandler
where
    T: std::fmt::Debug + Send + Sync + 'static,
{
    async fn handle_domain_event(&self, event: T) -> HandlerResult {
        println!("📨 [{}] Processing event: {:?}", self.name, event);
        Ok(())
    }
}

/// Create a boxed domain event handler
/// 
/// Example: `domain_handler!(MyHandler::new())`
#[macro_export]
macro_rules! domain_handler {
    ($handler:expr) => {
        Box::new($crate::events::handlers::DomainEventHandlerAdapter::new($handler))
    };
}

/// Create a simple logging handler for testing
/// 
/// Example: `logging_handler!("my-service")`
#[macro_export]
macro_rules! logging_handler {
    ($name:expr) => {
        $crate::domain_handler!($crate::events::handlers::LoggingEventHandler::new($name))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestEvent {
        message: String,
    }

    struct TestHandler;

    #[async_trait]
    impl DomainEventHandler<TestEvent> for TestHandler {
        async fn handle_domain_event(&self, event: TestEvent) -> HandlerResult {
            assert_eq!(event.message, "test message");
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_simple_handler() {
        let handler = TestHandler;
        
        let event = TestEvent {
            message: "test message".to_string(),
        };

        let result = handler.handle_domain_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logging_handler() {
        let handler = LoggingEventHandler::new("test-handler");
        
        let event = TestEvent {
            message: "test message".to_string(),
        };

        let result = handler.handle_domain_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_adapter() {
        let handler = DomainEventHandlerAdapter::new(TestHandler);
        
        let event = Event {
            id: "test-event-id".to_string(),
            topic: "test.topic".to_string(),
            metadata: tyl_pubsub_port::EventMetadata::new("test.service", "test.event"),
            headers: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
            payload: TestEvent {
                message: "test message".to_string(),
            },
        };

        let result = handler.handle(event).await;
        assert!(result.is_ok());
    }
}