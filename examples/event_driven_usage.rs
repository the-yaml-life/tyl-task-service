//! Event-driven microservice usage example
//!
//! This example demonstrates how to use the TYL microservice template
//! with event-driven architecture using tyl-pubsub-port.
//!
//! Run with: cargo run --example event_driven_usage

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tyl_task_service:{
    events::{EventService, DomainEventHandler},
    domain_handler, TaskServiceConfig,
};
use tyl_pubsub_port::{HandlerResult, HandlerError};
use async_trait::async_trait;
use uuid::Uuid;

// =============================================================================
// Example Domain Events
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProductCreated {
    product_id: Uuid,
    name: String,
    price: f64,
    category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderPlaced {
    order_id: Uuid,
    user_id: Uuid,
    product_id: Uuid,
    quantity: u32,
    total_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PaymentProcessed {
    payment_id: Uuid,
    order_id: Uuid,
    amount: f64,
    status: PaymentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum PaymentStatus {
    Success,
    Failed,
    Pending,
}

// =============================================================================
// Example Event Handlers
// =============================================================================

/// Handler for product creation events
/// Demonstrates simple business logic with validation
struct ProductCreatedHandler {
    service_name: String,
}

impl ProductCreatedHandler {
    fn new(service_name: String) -> Self {
        Self { service_name }
    }
}

#[async_trait]
impl DomainEventHandler<ProductCreated> for ProductCreatedHandler {
    async fn handle_domain_event(&self, event: ProductCreated) -> HandlerResult {
        println!(
            "🏷️  [{}] Processing product creation: {} (${:.2})",
            self.service_name, event.name, event.price
        );

        // Simulate business logic
        // 1. Validate product data
        // 2. Update inventory system
        // 3. Index product for search
        // 4. Send notifications
        
        sleep(Duration::from_millis(100)).await;
        
        println!("✅ Product {} indexed successfully", event.product_id);
        Ok(())
    }

    async fn validate_event(&self, event: &ProductCreated) -> Result<(), HandlerError> {
        if event.name.is_empty() {
            return Err(HandlerError::InvalidEventFormat {
                details: "Product name cannot be empty".to_string(),
            });
        }

        if event.price <= 0.0 {
            return Err(HandlerError::InvalidEventFormat {
                details: "Product price must be positive".to_string(),
            });
        }

        Ok(())
    }
}

/// Handler for order placement events
/// Demonstrates more complex business logic with external service calls
struct OrderPlacedHandler {
    service_name: String,
}

impl OrderPlacedHandler {
    fn new(service_name: String) -> Self {
        Self { service_name }
    }
}

#[async_trait]
impl DomainEventHandler<OrderPlaced> for OrderPlacedHandler {
    async fn handle_domain_event(&self, event: OrderPlaced) -> HandlerResult {
        println!(
            "🛒 [{}] Processing order: {} for user {} (${:.2})",
            self.service_name, event.order_id, event.user_id, event.total_amount
        );

        // Simulate business logic
        // 1. Check inventory availability
        // 2. Reserve items
        // 3. Calculate shipping
        // 4. Send confirmation email
        
        // Simulate external service call
        sleep(Duration::from_millis(200)).await;
        
        // Simulate inventory check
        if event.quantity > 10 {
            return Err(HandlerError::TemporaryFailure {
                reason: "Insufficient inventory, will retry when restocked".to_string(),
            });
        }

        println!("✅ Order {} processed successfully", event.order_id);
        Ok(())
    }
}

/// Handler for payment processing events
/// Demonstrates error handling and different response types
struct PaymentProcessedHandler {
    service_name: String,
}

impl PaymentProcessedHandler {
    fn new(service_name: String) -> Self {
        Self { service_name }
    }
}

#[async_trait]
impl DomainEventHandler<PaymentProcessed> for PaymentProcessedHandler {
    async fn handle_domain_event(&self, event: PaymentProcessed) -> HandlerResult {
        println!(
            "💳 [{}] Processing payment: {} for order {} (${:.2}) - Status: {:?}",
            self.service_name, event.payment_id, event.order_id, event.amount, event.status
        );

        match event.status {
            PaymentStatus::Success => {
                // Process successful payment
                // 1. Update order status
                // 2. Trigger fulfillment
                // 3. Send receipt
                sleep(Duration::from_millis(100)).await;
                println!("✅ Payment {} processed successfully", event.payment_id);
                Ok(())
            }
            PaymentStatus::Failed => {
                // Handle failed payment
                // 1. Update order status
                // 2. Send failure notification
                // 3. Offer retry options
                sleep(Duration::from_millis(50)).await;
                println!("❌ Payment {} failed", event.payment_id);
                
                // This is not an error in processing, just a business event
                Ok(())
            }
            PaymentStatus::Pending => {
                // Handle pending payment
                // 1. Set up monitoring
                // 2. Schedule follow-up checks
                sleep(Duration::from_millis(30)).await;
                println!("⏳ Payment {} is pending", event.payment_id);
                Ok(())
            }
        }
    }
}

// =============================================================================
// Main Example
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for better logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Event-Driven Microservice Example");
    println!("================================================\n");

    // Initialize the event service
    let event_service = EventService::new().await?;
    let service_name = "example-microservice".to_string();

    // =======================================================================
    // Step 1: Set up event handlers (subscribers)
    // =======================================================================
    
    println!("📡 Setting up event handlers...");

    // Subscribe to product events
    let product_handler = domain_handler!(ProductCreatedHandler::new(service_name.clone()));
    let product_subscription = event_service
        .subscribe("product.created", product_handler)
        .await?;
    println!("   ✓ Product handler subscribed (ID: {})", product_subscription);

    // Subscribe to order events
    let order_handler = domain_handler!(OrderPlacedHandler::new(service_name.clone()));
    let order_subscription = event_service
        .subscribe("order.placed", order_handler)
        .await?;
    println!("   ✓ Order handler subscribed (ID: {})", order_subscription);

    // Subscribe to payment events
    let payment_handler = domain_handler!(PaymentProcessedHandler::new(service_name.clone()));
    let payment_subscription = event_service
        .subscribe("payment.processed", payment_handler)
        .await?;
    println!("   ✓ Payment handler subscribed (ID: {})", payment_subscription);

    println!("✅ All handlers set up successfully!\n");

    // Give handlers a moment to initialize
    sleep(Duration::from_millis(100)).await;

    // =======================================================================
    // Step 2: Publish events (simulate business operations)
    // =======================================================================

    println!("📤 Publishing events...");

    // Publish product creation event
    let product = ProductCreated {
        product_id: Uuid::new_v4(),
        name: "Wireless Headphones".to_string(),
        price: 99.99,
        category: "Electronics".to_string(),
    };
    let product_event_id = event_service.publish("product.created", product.clone()).await?;
    println!("   📦 Published product creation event (ID: {})", product_event_id);

    // Wait a bit for processing
    sleep(Duration::from_millis(200)).await;

    // Publish order placement event
    let order = OrderPlaced {
        order_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        product_id: product.product_id,
        quantity: 2,
        total_amount: 199.98,
    };
    let order_event_id = event_service.publish("order.placed", order.clone()).await?;
    println!("   🛒 Published order placement event (ID: {})", order_event_id);

    // Wait a bit for processing
    sleep(Duration::from_millis(300)).await;

    // Publish successful payment event
    let payment_success = PaymentProcessed {
        payment_id: Uuid::new_v4(),
        order_id: order.order_id,
        amount: order.total_amount,
        status: PaymentStatus::Success,
    };
    let payment_event_id = event_service.publish("payment.processed", payment_success).await?;
    println!("   💳 Published successful payment event (ID: {})", payment_event_id);

    // Wait a bit for processing
    sleep(Duration::from_millis(200)).await;

    // Publish failed payment event (to show error handling)
    let payment_failed = PaymentProcessed {
        payment_id: Uuid::new_v4(),
        order_id: Uuid::new_v4(),
        amount: 50.0,
        status: PaymentStatus::Failed,
    };
    let failed_payment_event_id = event_service.publish("payment.processed", payment_failed).await?;
    println!("   💳 Published failed payment event (ID: {})", failed_payment_event_id);

    // Wait for all processing to complete
    sleep(Duration::from_millis(300)).await;

    // =======================================================================
    // Step 3: Demonstrate batch publishing
    // =======================================================================

    println!("\n📤 Publishing batch events...");

    // Create multiple products at once
    let products = vec![
        ProductCreated {
            product_id: Uuid::new_v4(),
            name: "Smart Watch".to_string(),
            price: 199.99,
            category: "Electronics".to_string(),
        },
        ProductCreated {
            product_id: Uuid::new_v4(),
            name: "Coffee Mug".to_string(),
            price: 12.99,
            category: "Kitchen".to_string(),
        },
        ProductCreated {
            product_id: Uuid::new_v4(),
            name: "Notebook".to_string(),
            price: 8.99,
            category: "Office".to_string(),
        },
    ];

    for (i, product) in products.into_iter().enumerate() {
        let event_id = event_service.publish("product.created", product).await?;
        println!("   📦 Published batch product {} (ID: {})", i + 1, event_id);
        sleep(Duration::from_millis(50)).await; // Stagger the events
    }

    // Wait for batch processing
    sleep(Duration::from_millis(500)).await;

    // =======================================================================
    // Step 4: Demonstrate error handling with invalid event
    // =======================================================================

    println!("\n⚠️  Testing error handling...");

    // Try to publish invalid product (empty name, negative price)
    let invalid_product = ProductCreated {
        product_id: Uuid::new_v4(),
        name: "".to_string(), // Invalid: empty name
        price: -10.0,         // Invalid: negative price
        category: "Test".to_string(),
    };

    match event_service.publish("product.created", invalid_product).await {
        Ok(event_id) => {
            println!("   📦 Published invalid product event (ID: {})", event_id);
            println!("   ⚠️  Note: Validation happens at handler level, not publish level");
        }
        Err(e) => {
            println!("   ❌ Failed to publish invalid product: {}", e);
        }
    }

    // Wait for error processing
    sleep(Duration::from_millis(200)).await;

    // =======================================================================
    // Cleanup and summary
    // =======================================================================

    println!("\n🧹 Cleaning up...");

    // Unsubscribe from events
    event_service.unsubscribe(product_subscription).await?;
    event_service.unsubscribe(order_subscription).await?;
    event_service.unsubscribe(payment_subscription).await?;

    println!("   ✓ All subscriptions cancelled");

    println!("\n🎉 Event-driven microservice example completed!");
    println!("\nKey takeaways:");
    println!("   📡 Easy event subscription with type-safe handlers");
    println!("   📤 Simple event publishing with automatic serialization");
    println!("   🔧 Built-in error handling and validation");
    println!("   🎯 Clean separation between business logic and event infrastructure");
    println!("   🚀 Ready for production with real adapters (Redis, Kafka, etc.)");

    Ok(())
}

// =============================================================================
// Additional Examples for Copy-Paste Usage
// =============================================================================

/// Example of how to integrate event handling into HTTP handlers
/// 
/// ```rust
/// use axum::{extract::State, Json};
/// use tyl_microservice::AppState;
/// 
/// async fn create_product_handler(
///     State(state): State<AppState>,
///     Json(request): Json<CreateProductRequest>,
/// ) -> Result<Json<CreateProductResponse>, ApiError> {
///     // 1. Validate request
///     // 2. Create product in database
///     let product = create_product_in_db(request).await?;
///     
///     // 3. Publish domain event
///     let event = ProductCreated {
///         product_id: product.id,
///         name: product.name.clone(),
///         price: product.price,
///         category: product.category.clone(),
///     };
///     
///     state.event_service.publish("product.created", event).await
///         .map_err(|e| ApiError::Internal(format!("Failed to publish event: {}", e)))?;
///     
///     // 4. Return response
///     Ok(Json(CreateProductResponse { product }))
/// }
/// ```

/// Example of setting up event handlers during application startup
/// 
/// ```rust
/// async fn setup_event_handlers(event_service: &EventService) -> Result<(), Box<dyn std::error::Error>> {
///     // Set up all domain event handlers
///     event_service.subscribe("product.created", 
///         domain_handler!(ProductCreatedHandler::new("product-service".to_string()))).await?;
///     
///     event_service.subscribe("order.placed", 
///         domain_handler!(OrderPlacedHandler::new("order-service".to_string()))).await?;
///     
///     event_service.subscribe("payment.processed", 
///         domain_handler!(PaymentProcessedHandler::new("payment-service".to_string()))).await?;
///     
///     Ok(())
/// }
/// ```
fn _example_integration_doc() {}