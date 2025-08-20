//! HTTP client adapter for external service integrations
//!
//! This module provides HTTP client functionality for communicating with external services.

use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, error};

use crate::{
    config::ExternalConfig,
    TaskServiceResult, TaskServiceError,
    utils::generate_correlation_id,
};

/// HTTP client manager for external service calls
pub struct HttpClientManager {
    client: Client,
    config: ExternalConfig,
}

/// External service response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalServiceResponse<T> {
    pub data: T,
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// External service request wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalServiceRequest<T> {
    pub data: T,
    pub correlation_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ExternalServiceRequest<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            correlation_id: generate_correlation_id(),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl HttpClientManager {
    /// Create a new HTTP client manager
    pub fn new(config: ExternalConfig) -> TaskServiceResult<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_millis(config.timeout_ms))
            .user_agent(format!("tyl-microservice/1.0"))
            .build()
            .map_err(|e| TaskServiceError::ExternalService {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self { client, config })
    }

    /// Make a GET request to an external service
    pub async fn get<T>(&self, url: &str) -> TaskServiceResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let correlation_id = generate_correlation_id();
        
        info!(
            correlation_id = %correlation_id,
            url = %url,
            "Making GET request to external service"
        );

        let response = self
            .client
            .get(url)
            .header("X-Correlation-ID", &correlation_id)
            .send()
            .await
            .map_err(|e| {
                error!(
                    correlation_id = %correlation_id,
                    url = %url,
                    error = %e,
                    "GET request failed"
                );
                TaskServiceError::ExternalService {
                    message: format!("GET request to {} failed: {}", url, e),
                }
            })?;

        if !response.status().is_success() {
            error!(
                correlation_id = %correlation_id,
                url = %url,
                status = %response.status(),
                "GET request returned error status"
            );
            return Err(TaskServiceError::ExternalService {
                message: format!("GET request to {} failed with status: {}", url, response.status()),
            });
        }

        let result = response
            .json::<T>()
            .await
            .map_err(|e| {
                error!(
                    correlation_id = %correlation_id,
                    url = %url,
                    error = %e,
                    "Failed to parse response JSON"
                );
                TaskServiceError::ExternalService {
                    message: format!("Failed to parse response from {}: {}", url, e),
                }
            })?;

        info!(
            correlation_id = %correlation_id,
            url = %url,
            "GET request completed successfully"
        );

        Ok(result)
    }

    /// Make a POST request to an external service
    pub async fn post<T, R>(&self, url: &str, payload: &T) -> TaskServiceResult<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let correlation_id = generate_correlation_id();
        
        info!(
            correlation_id = %correlation_id,
            url = %url,
            "Making POST request to external service"
        );

        let request = ExternalServiceRequest::new(payload);

        let response = self
            .client
            .post(url)
            .header("X-Correlation-ID", &correlation_id)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                error!(
                    correlation_id = %correlation_id,
                    url = %url,
                    error = %e,
                    "POST request failed"
                );
                TaskServiceError::ExternalService {
                    message: format!("POST request to {} failed: {}", url, e),
                }
            })?;

        if !response.status().is_success() {
            error!(
                correlation_id = %correlation_id,
                url = %url,
                status = %response.status(),
                "POST request returned error status"
            );
            return Err(TaskServiceError::ExternalService {
                message: format!("POST request to {} failed with status: {}", url, response.status()),
            });
        }

        let result = response
            .json::<R>()
            .await
            .map_err(|e| {
                error!(
                    correlation_id = %correlation_id,
                    url = %url,
                    error = %e,
                    "Failed to parse response JSON"
                );
                TaskServiceError::ExternalService {
                    message: format!("Failed to parse response from {}: {}", url, e),
                }
            })?;

        info!(
            correlation_id = %correlation_id,
            url = %url,
            "POST request completed successfully"
        );

        Ok(result)
    }

    /// Make a request with retry logic
    pub async fn get_with_retry<T>(&self, url: &str) -> TaskServiceResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut last_error = None;
        
        for attempt in 1..=self.config.retry_attempts {
            match self.get(url).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < self.config.retry_attempts {
                        info!(
                            attempt = attempt,
                            max_attempts = self.config.retry_attempts,
                            url = %url,
                            "Request failed, retrying..."
                        );
                        
                        tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| TaskServiceError::ExternalService {
            message: "All retry attempts failed".to_string(),
        }))
    }

    /// Health check for external service connectivity
    pub async fn health_check(&self, health_url: &str) -> TaskServiceResult<bool> {
        match self.client.get(health_url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

/// Example external service client
/// Replace with your actual external service integrations
pub struct ExampleServiceClient {
    http_client: HttpClientManager,
    base_url: String,
}

impl ExampleServiceClient {
    pub fn new(base_url: String, config: ExternalConfig) -> TaskServiceResult<Self> {
        let http_client = HttpClientManager::new(config)?;
        
        Ok(Self {
            http_client,
            base_url,
        })
    }

    /// Example method for calling external service
    pub async fn get_task_data(&self, task_id: &str) -> TaskServiceResult<TaskData> {
        let url = format!("{}/tasks/{}", self.base_url, task_id);
        self.http_client.get(&url).await
    }

    /// Example method for posting data to external service
    pub async fn create_task(&self, task: &ExternalCreateTaskRequest) -> TaskServiceResult<TaskData> {
        let url = format!("{}/tasks", self.base_url);
        self.http_client.post(&url, task).await
    }

    /// Health check for the external service
    pub async fn health_check(&self) -> TaskServiceResult<bool> {
        let health_url = format!("{}/health", self.base_url);
        self.http_client.health_check(&health_url).await
    }
}

/// Example data structures for external service communication
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskData {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalCreateTaskRequest {
    pub name: String,
    pub email: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ExternalConfig {
        ExternalConfig {
            timeout_ms: 5000,
            retry_attempts: 3,
            retry_delay_ms: 1000,
        }
    }

    #[tokio::test]
    async fn test_http_client_creation() {
        let config = create_test_config();
        let client = HttpClientManager::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_external_service_request_wrapper() {
        let data = ExternalCreateTaskRequest {
            name: "Test Task".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let request = ExternalServiceRequest::new(&data);
        assert!(!request.correlation_id.is_empty());
        assert_eq!(request.data.name, "Test Task");
    }

    // Note: Add integration tests with a test HTTP server for more comprehensive testing
}