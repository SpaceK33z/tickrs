//! Domain models for TickTick data structures.
//!
//! This module contains the core data types used throughout the application,
//! matching the TickTick API's JSON format with serde serialization support.
//!
//! # Main Types
//!
//! - [`Task`] - A task/to-do item with title, dates, priority, tags, etc.
//! - [`Project`] - A project/list that contains tasks
//! - [`ChecklistItem`] - A subtask within a task
//!
//! # Enums
//!
//! - [`Priority`] - Task priority levels (None, Low, Medium, High)
//! - [`Status`] - Task completion status (Normal, Complete)
//!
//! # Constants
//!
//! - [`INBOX_PROJECT_ID`] - The special ID for the Inbox project

pub mod priority;
pub mod project;
pub mod status;
pub mod subtask;
pub mod task;
pub mod time;

pub use priority::Priority;
pub use project::{Project, ProjectData, INBOX_PROJECT_ID};
pub use status::Status;
pub use subtask::ChecklistItem;
pub use task::Task;
