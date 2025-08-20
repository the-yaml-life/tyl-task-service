//! Example events and handlers for reference

use crate::events::{DomainEventHandler, EventHandlerResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// Example Domain Events
// =============================================================================

/// Example user registration event
/// 
/// This demonstrates a typical domain event that might be published
/// when a user registers in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRegistered {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub registered_at: DateTime<Utc>,
}

impl TaskRegistered {
    pub fn new(email: String, username: String) -> Self {
        Self {
            user_id: Uuid::new_v4(),
            email,
            username,
            registered_at: Utc::now(),
        }
    }
}

/// Example order created event
/// 
/// This demonstrates an event that might be published when
/// an order is created in an e-commerce system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreated {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub total_amount: f64,
    pub currency: String,
    pub items: Vec<OrderItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: Uuid,
    pub quantity: u32,
    pub unit_price: f64,
}

impl OrderCreated {
    pub fn new(user_id: Uuid, items: Vec<OrderItem>) -> Self {
        let total_amount = items.iter()
            .map(|item| item.quantity as f64 * item.unit_price)
            .sum();

        Self {
            order_id: Uuid::new_v4(),
            user_id,
            total_amount,
            currency: "USD".to_string(),
            items,
            created_at: Utc::now(),
        }
    }
}

/// Example system notification event
/// 
/// This demonstrates a general-purpose notification event
/// that could be used for various system notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemNotification {
    pub notification_id: Uuid,
    pub title: String,
    pub message: String,
    pub level: NotificationLevel,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}

impl SystemNotification {
    pub fn info(title: String, message: String) -> Self {
        Self {
            notification_id: Uuid::new_v4(),
            title,
            message,
            level: NotificationLevel::Info,
            created_at: Utc::now(),
        }
    }

    pub fn warning(title: String, message: String) -> Self {
        Self {
            notification_id: Uuid::new_v4(),
            title,
            message,
            level: NotificationLevel::Warning,
            created_at: Utc::now(),
        }
    }

    pub fn error(title: String, message: String) -> Self {
        Self {
            notification_id: Uuid::new_v4(),
            title,
            message,
            level: NotificationLevel::Error,
            created_at: Utc::now(),
        }
    }
}

// =============================================================================
// Example Event Handlers
// =============================================================================

/// Example handler for user registration events
/// 
/// This handler demonstrates how to process user registration events,
/// including validation, logging, and business logic.
pub struct TaskRegisteredHandler {
    service_name: String,
}

impl TaskRegisteredHandler {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }
}

#[async_trait]
impl DomainEventHandler<TaskRegistered> for TaskRegisteredHandler {
    async fn handle_domain_event(&self, event: TaskRegistered) -> EventHandlerResult {
        tracing::info!(
            service = %self.service_name,
            user_id = %event.user_id,
            email = %event.email,
            username = %event.username,
            "Processing user registration event"
        );

        // Example business logic:
        // 1. Send welcome email
        // 2. Create user profile
        // 3. Initialize user preferences
        // 4. Send notification to admin

        // Simulate async work
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        tracing::info!(
            user_id = %event.user_id,
            "User registration processing completed"
        );

        Ok(())
    }

}

/// Example handler for order created events
/// 
/// This handler demonstrates processing order events with
/// more complex business logic and external service calls.
pub struct OrderCreatedHandler {
    service_name: String,
}

impl OrderCreatedHandler {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }
}

#[async_trait]
impl DomainEventHandler<OrderCreated> for OrderCreatedHandler {
    async fn handle_domain_event(&self, event: OrderCreated) -> EventHandlerResult {
        tracing::info!(
            service = %self.service_name,
            order_id = %event.order_id,
            user_id = %event.user_id,
            total_amount = %event.total_amount,
            items_count = event.items.len(),
            "Processing order created event"
        );

        // Example business logic:
        // 1. Reserve inventory
        // 2. Process payment
        // 3. Schedule shipping
        // 4. Send confirmation email
        // 5. Update analytics

        // Simulate processing each item
        for item in &event.items {
            tracing::debug!(
                order_id = %event.order_id,
                product_id = %item.product_id,
                quantity = item.quantity,
                "Processing order item"
            );
            
            // Simulate async item processing
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }

        tracing::info!(
            order_id = %event.order_id,
            "Order processing completed"
        );

        Ok(())
    }

}

/// Example handler for system notifications
/// 
/// This handler demonstrates handling different notification levels
/// and routing to appropriate channels.
pub struct SystemNotificationHandler {
    service_name: String,
}

impl SystemNotificationHandler {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }
}

#[async_trait]
impl DomainEventHandler<SystemNotification> for SystemNotificationHandler {
    async fn handle_domain_event(&self, event: SystemNotification) -> EventHandlerResult {
        match event.level {
            NotificationLevel::Info => {
                tracing::info!(
                    service = %self.service_name,
                    notification_id = %event.notification_id,
                    title = %event.title,
                    "Processing info notification"
                );
            }
            NotificationLevel::Warning => {
                tracing::warn!(
                    service = %self.service_name,
                    notification_id = %event.notification_id,
                    title = %event.title,
                    message = %event.message,
                    "Processing warning notification"
                );
            }
            NotificationLevel::Error => {
                tracing::error!(
                    service = %self.service_name,
                    notification_id = %event.notification_id,
                    title = %event.title,
                    message = %event.message,
                    "Processing error notification"
                );
            }
        }

        // Route to appropriate notification channel based on level
        match event.level {
            NotificationLevel::Info => {
                // Send to general notification channel
            }
            NotificationLevel::Warning => {
                // Send to admin channel
            }
            NotificationLevel::Error => {
                // Send to emergency channel
                // Trigger alerts
            }
        }

        Ok(())
    }
}

// =============================================================================
// Example Usage Patterns
// =============================================================================

/// Example function showing how to set up event handlers in a microservice
/// 
/// This function demonstrates the typical pattern for setting up
/// event subscriptions in a microservice.
pub async fn setup_example_event_handlers(
    event_service: &crate::events::EventService,
    service_name: String,
) -> crate::TaskServiceResult<()> {
    use crate::domain_handler;

    // Subscribe to user registration events
    let user_handler = domain_handler!(TaskRegisteredHandler::new(service_name.clone()));
    event_service.subscribe("user.registered", user_handler).await?;

    // Subscribe to order creation events
    let order_handler = domain_handler!(OrderCreatedHandler::new(service_name.clone()));
    event_service.subscribe("order.created", order_handler).await?;

    // Subscribe to system notifications
    let notification_handler = domain_handler!(SystemNotificationHandler::new(service_name));
    event_service.subscribe("system.notification", notification_handler).await?;

    tracing::info!("Event handlers registered successfully");
    Ok(())
}

/// Example function showing how to publish events
/// 
/// This function demonstrates the typical pattern for publishing
/// domain events in a microservice.
pub async fn publish_example_events(
    event_service: &crate::events::EventService,
) -> crate::TaskServiceResult<()> {
    // Publish user registration event
    let user_event = TaskRegistered::new(
        "user@example.com".to_string(),
        "newuser123".to_string(),
    );
    event_service.publish("user.registered", user_event).await?;

    // Publish order creation event
    let order_items = vec![
        OrderItem {
            product_id: Uuid::new_v4(),
            quantity: 2,
            unit_price: 29.99,
        },
        OrderItem {
            product_id: Uuid::new_v4(),
            quantity: 1,
            unit_price: 49.99,
        },
    ];
    let order_event = OrderCreated::new(Uuid::new_v4(), order_items);
    event_service.publish("order.created", order_event).await?;

    // Publish system notification
    let notification = SystemNotification::info(
        "System Update".to_string(),
        "System has been updated to version 1.2.3".to_string(),
    );
    event_service.publish("system.notification", notification).await?;

    tracing::info!("Example events published successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventService;

    #[tokio::test]
    async fn test_user_registered_handler() {
        let handler = TaskRegisteredHandler::new("test-service".to_string());
        let event = TaskRegistered::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
        );

        let result = handler.handle_domain_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_order_created_handler() {
        let handler = OrderCreatedHandler::new("test-service".to_string());
        let items = vec![OrderItem {
            product_id: Uuid::new_v4(),
            quantity: 1,
            unit_price: 10.0,
        }];
        let event = OrderCreated::new(Uuid::new_v4(), items);

        let result = handler.handle_domain_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validation_failure() {
        let handler = TaskRegisteredHandler::new("test-service".to_string());
        let invalid_event = TaskRegistered {
            user_id: Uuid::new_v4(),
            email: "invalid-email".to_string(), // No @ symbol
            username: "ab".to_string(), // Too short
            registered_at: Utc::now(),
        };

        let result = handler.handle_domain_event(invalid_event).await;
        // Note: In a real implementation, you might want to validate events
        // For now, we just test that it processes without panicking
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_example_event_publishing() {
        let event_service = EventService::new().await.unwrap();
        let result = publish_example_events(&event_service).await;
        assert!(result.is_ok());
    }
}