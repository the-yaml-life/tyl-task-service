//! Event service for publishing and managing events

use crate::{TaskServiceError, TaskServiceResult};
use serde::Serialize;
use std::sync::Arc;
use tyl_pubsub_port::{EventPublisher, EventSubscriber, EventHandler, MockPubSubAdapter, EventId, SubscriptionId};

/// Event service that provides publishing and subscription capabilities
/// 
/// This service acts as a facade over tyl-pubsub-port, providing
/// microservice-specific functionality and error handling.
pub struct EventService<A = MockPubSubAdapter> 
where 
    A: EventPublisher + EventSubscriber + Send + Sync + 'static,
{
    adapter: Arc<A>,
}

impl EventService<MockPubSubAdapter> {
    /// Create a new event service with the default mock adapter
    /// 
    /// In production, this would be replaced with a real adapter (Redis, Kafka, etc.)
    pub async fn new() -> TaskServiceResult<Self> {
        let adapter = Arc::new(MockPubSubAdapter::new());
        
        Ok(Self {
            adapter,
        })
    }
}

impl<A> EventService<A> 
where 
    A: EventPublisher + EventSubscriber + Send + Sync + 'static,
{
    /// Create an event service with a custom adapter
    pub fn with_adapter(adapter: Arc<A>) -> Self {
        Self {
            adapter,
        }
    }

    /// Publish an event to a topic
    /// 
    /// # Arguments
    /// * `topic` - The topic to publish to (e.g., "user.registered", "order.created")
    /// * `event` - The event payload to publish
    /// 
    /// # Example
    /// ```rust
    /// # use tyl_microservice::events::EventService;
    /// # use serde::{Serialize, Deserialize};
    /// # 
    /// #[derive(Serialize, Deserialize)]
    /// struct UserRegistered {
    ///     user_id: String,
    ///     email: String,
    /// }
    /// 
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let event_service = EventService::new().await?;
    /// 
    /// let event = UserRegistered {
    ///     user_id: "123".to_string(),
    ///     email: "user@example.com".to_string(),
    /// };
    /// 
    /// let event_id = event_service.publish("user.registered", event).await?;
    /// println!("Published event: {}", event_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn publish<T>(&self, topic: &str, event: T) -> TaskServiceResult<EventId>
    where
        T: Serialize + Send + Sync,
    {
        self.adapter
            .publish(topic, event)
            .await
            .map_err(|e| TaskServiceError::ExternalService {
                message: format!("Failed to publish event to topic '{}': {}", topic, e),
            })
    }

    /// Publish an event with a partition key for ordered processing
    pub async fn publish_with_key<T>(&self, topic: &str, key: &str, event: T) -> TaskServiceResult<EventId>
    where
        T: Serialize + Send + Sync,
    {
        self.adapter
            .publish_with_key(topic, key, event)
            .await
            .map_err(|e| TaskServiceError::ExternalService {
                message: format!("Failed to publish keyed event to topic '{}' with key '{}': {}", topic, key, e),
            })
    }

    /// Subscribe to a topic with an event handler
    /// 
    /// # Arguments
    /// * `topic` - The topic to subscribe to
    /// * `handler` - The event handler that will process events
    /// 
    /// # Example
    /// ```rust
    /// # use tyl_microservice::events::{EventService, DomainEventHandler};
    /// # use tyl_pubsub_port::{Event, HandlerResult};
    /// # use serde::{Serialize, Deserialize};
    /// # use async_trait::async_trait;
    /// 
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct UserRegistered {
    ///     user_id: String,
    ///     email: String,
    /// }
    /// 
    /// struct UserRegisteredHandler;
    /// 
    /// #[async_trait]
    /// impl DomainEventHandler<UserRegistered> for UserRegisteredHandler {
    ///     async fn handle_domain_event(&self, event: UserRegistered) -> HandlerResult {
    ///         println!("User registered: {:?}", event);
    ///         Ok(())
    ///     }
    /// }
    /// 
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let event_service = EventService::new().await?;
    /// let handler = Box::new(UserRegisteredHandler);
    /// 
    /// let subscription_id = event_service.subscribe("user.registered", handler).await?;
    /// println!("Subscribed with ID: {}", subscription_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe<T, H>(&self, topic: &str, handler: Box<H>) -> TaskServiceResult<SubscriptionId>
    where
        T: serde::de::DeserializeOwned + Send + Sync + 'static,
        H: EventHandler<T> + 'static,
    {
        self.adapter
            .subscribe(topic, handler)
            .await
            .map_err(|e| TaskServiceError::ExternalService {
                message: format!("Failed to subscribe to topic '{}': {}", topic, e),
            })
    }

    /// Unsubscribe from a topic
    pub async fn unsubscribe(&self, subscription_id: SubscriptionId) -> TaskServiceResult<()> {
        self.adapter
            .unsubscribe(subscription_id)
            .await
            .map_err(|e| TaskServiceError::ExternalService {
                message: format!("Failed to unsubscribe: {}", e),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestEvent {
        message: String,
    }

    #[tokio::test]
    async fn test_event_service_creation() {
        let service = EventService::new().await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_publish_event() {
        let service = EventService::new().await.unwrap();
        
        let event = TestEvent {
            message: "Hello, World!".to_string(),
        };

        let result = service.publish("test.events", event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_publish_with_key() {
        let service = EventService::new().await.unwrap();
        
        let event = TestEvent {
            message: "Keyed event".to_string(),
        };

        let result = service.publish_with_key("test.events", "key1", event).await;
        assert!(result.is_ok());
    }
}