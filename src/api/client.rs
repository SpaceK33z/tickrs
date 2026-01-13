use anyhow::{Context, Result};
use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use tracing::{debug, instrument};

use crate::config::TokenStorage;

/// Base URL for TickTick Open API
pub const API_BASE_URL: &str = "https://api.ticktick.com/open/v1";

/// TickTick API client wrapper
#[derive(Debug, Clone)]
pub struct TickTickClient {
    client: Client,
    token: String,
}

/// API error response from TickTick
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Authentication required. Run 'tickrs init' to authenticate.")]
    NotAuthenticated,

    #[error("Invalid or expired token. Run 'tickrs init' to re-authenticate.")]
    Unauthorized,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Rate limited. Please wait and try again.")]
    RateLimited,

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

impl TickTickClient {
    /// Create a new client with the stored token
    pub fn new() -> Result<Self> {
        let token = TokenStorage::load()?.ok_or(ApiError::NotAuthenticated)?;

        Self::with_token(token)
    }

    /// Create a new client with a specific token
    pub fn with_token(token: String) -> Result<Self> {
        let client = Client::builder()
            .user_agent(format!("tickrs/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, token })
    }

    /// Build the full URL for an endpoint
    fn url(&self, endpoint: &str) -> String {
        format!("{}{}", API_BASE_URL, endpoint)
    }

    /// Make a GET request to the API
    #[instrument(skip(self), fields(endpoint = %endpoint))]
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, ApiError> {
        debug!("GET {}", endpoint);

        let response = self
            .client
            .get(self.url(endpoint))
            .bearer_auth(&self.token)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request to the API with JSON body
    #[instrument(skip(self, body), fields(endpoint = %endpoint))]
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        debug!("POST {}", endpoint);

        let response = self
            .client
            .post(self.url(endpoint))
            .bearer_auth(&self.token)
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request without a body (for actions like complete)
    #[instrument(skip(self), fields(endpoint = %endpoint))]
    pub async fn post_empty<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, ApiError> {
        debug!("POST {} (empty body)", endpoint);

        let response = self
            .client
            .post(self.url(endpoint))
            .bearer_auth(&self.token)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a DELETE request to the API
    #[instrument(skip(self), fields(endpoint = %endpoint))]
    pub async fn delete(&self, endpoint: &str) -> Result<(), ApiError> {
        debug!("DELETE {}", endpoint);

        let response = self
            .client
            .delete(self.url(endpoint))
            .bearer_auth(&self.token)
            .send()
            .await?;

        self.handle_empty_response(response).await
    }

    /// Handle API response and parse JSON
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T, ApiError> {
        let status = response.status();
        let url = response.url().to_string();

        match status {
            StatusCode::OK | StatusCode::CREATED => {
                let text = response.text().await?;
                debug!("Response: {}", &text[..text.len().min(500)]);
                serde_json::from_str(&text).map_err(|e| {
                    ApiError::ParseError(format!("{}: {}", e, &text[..text.len().min(200)]))
                })
            }
            StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized),
            StatusCode::NOT_FOUND => Err(ApiError::NotFound(url)),
            StatusCode::BAD_REQUEST => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::BadRequest(text))
            }
            StatusCode::TOO_MANY_REQUESTS => Err(ApiError::RateLimited),
            _ if status.is_server_error() => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::ServerError(format!("{}: {}", status, text)))
            }
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::ServerError(format!(
                    "Unexpected status {}: {}",
                    status, text
                )))
            }
        }
    }

    /// Handle API response for endpoints that return empty body
    async fn handle_empty_response(&self, response: Response) -> Result<(), ApiError> {
        let status = response.status();
        let url = response.url().to_string();

        match status {
            StatusCode::OK | StatusCode::NO_CONTENT => Ok(()),
            StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized),
            StatusCode::NOT_FOUND => Err(ApiError::NotFound(url)),
            StatusCode::BAD_REQUEST => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::BadRequest(text))
            }
            StatusCode::TOO_MANY_REQUESTS => Err(ApiError::RateLimited),
            _ if status.is_server_error() => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::ServerError(format!("{}: {}", status, text)))
            }
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::ServerError(format!(
                    "Unexpected status {}: {}",
                    status, text
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_building() {
        // Create client with dummy token (won't make real requests)
        let client = TickTickClient::with_token("test_token".to_string()).unwrap();

        assert_eq!(
            client.url("/project"),
            "https://api.ticktick.com/open/v1/project"
        );
        assert_eq!(
            client.url("/project/123/task/456"),
            "https://api.ticktick.com/open/v1/project/123/task/456"
        );
    }

    #[test]
    fn test_api_error_display() {
        assert_eq!(
            ApiError::NotAuthenticated.to_string(),
            "Authentication required. Run 'tickrs init' to authenticate."
        );
        assert_eq!(
            ApiError::Unauthorized.to_string(),
            "Invalid or expired token. Run 'tickrs init' to re-authenticate."
        );
        assert_eq!(
            ApiError::NotFound("/project/123".to_string()).to_string(),
            "Resource not found: /project/123"
        );
    }
}
