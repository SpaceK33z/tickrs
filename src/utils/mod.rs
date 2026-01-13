//! Utility modules for tickrs
//!
//! This module contains shared utilities including:
//! - Date parsing for natural language dates
//! - Error types and conversions

pub mod date_parser;
pub mod error;

pub use date_parser::{format_datetime, local_timezone, parse_date, parse_date_with_timezone, parse_future_date, DateParseError};
pub use error::{AppError, ErrorCode};
