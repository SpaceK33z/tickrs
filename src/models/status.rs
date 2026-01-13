use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Task completion status as used by TickTick API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    /// Normal/incomplete task (API value: 0)
    #[default]
    Normal,
    /// Completed task (API value: 2)
    Complete,
}

impl Status {
    /// Convert status to TickTick API integer value
    pub fn to_api_value(self) -> i32 {
        match self {
            Status::Normal => 0,
            Status::Complete => 2,
        }
    }

    /// Create status from TickTick API integer value
    pub fn from_api_value(value: i32) -> Self {
        match value {
            2 => Status::Complete,
            _ => Status::Normal,
        }
    }

    /// Check if the task is complete
    pub fn is_complete(self) -> bool {
        matches!(self, Status::Complete)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Normal => write!(f, "incomplete"),
            Status::Complete => write!(f, "complete"),
        }
    }
}

impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.to_api_value())
    }
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i32::deserialize(deserializer)?;
        Ok(Status::from_api_value(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_api_values() {
        assert_eq!(Status::Normal.to_api_value(), 0);
        assert_eq!(Status::Complete.to_api_value(), 2);
    }

    #[test]
    fn test_status_from_api_values() {
        assert_eq!(Status::from_api_value(0), Status::Normal);
        assert_eq!(Status::from_api_value(2), Status::Complete);
        assert_eq!(Status::from_api_value(99), Status::Normal); // Unknown defaults to Normal
    }

    #[test]
    fn test_status_is_complete() {
        assert!(!Status::Normal.is_complete());
        assert!(Status::Complete.is_complete());
    }

    #[test]
    fn test_status_serialization() {
        let status = Status::Complete;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "2");
    }

    #[test]
    fn test_status_deserialization() {
        let status: Status = serde_json::from_str("0").unwrap();
        assert_eq!(status, Status::Normal);
    }
}
