//! TickTick API client and related types.
//!
//! This module provides the HTTP client for interacting with the TickTick Open API,
//! including OAuth authentication, project management, and task operations.
//!
//! # Main Types
//!
//! - [`TickTickClient`] - The main API client for making authenticated requests
//! - [`AuthHandler`] - Handles OAuth 2.0 authentication flow
//! - [`ApiError`] - Error types returned by API operations
//!
//! # Request Types
//!
//! - [`CreateProjectRequest`] / [`UpdateProjectRequest`] - Project creation/update
//! - [`CreateTaskRequest`] / [`UpdateTaskRequest`] - Task creation/update
//!
//! # Example
//!
//! ```no_run
//! use ticktickrs::api::{TickTickClient, CreateTaskRequest};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = TickTickClient::with_token("your_token".to_string())?;
//! let request = CreateTaskRequest {
//!     title: "My task".to_string(),
//!     project_id: "inbox".to_string(),
//!     content: None,
//!     is_all_day: None,
//!     start_date: None,
//!     due_date: None,
//!     priority: None,
//!     time_zone: None,
//!     tags: None,
//!     items: None,
//! };
//! let task = client.create_task(&request).await?;
//! # Ok(())
//! # }
//! ```

pub mod auth;
pub mod client;
pub mod project;
pub mod task;
pub mod types;

pub use auth::AuthHandler;
pub use client::{ApiError, TickTickClient};
pub use project::{CreateProjectRequest, UpdateProjectRequest};
pub use task::{CreateTaskRequest, UpdateTaskRequest};
