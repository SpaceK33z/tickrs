use std::fmt;
use std::str::FromStr;

use clap::ValueEnum;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Task priority levels as used by TickTick API
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum Priority {
    #[default]
    None,
    Low,
    Medium,
    High,
}

impl Priority {
    /// Convert priority to TickTick API integer value
    pub fn to_api_value(self) -> i32 {
        match self {
            Priority::None => 0,
            Priority::Low => 1,
            Priority::Medium => 3,
            Priority::High => 5,
        }
    }

    /// Create priority from TickTick API integer value
    pub fn from_api_value(value: i32) -> Self {
        match value {
            1 => Priority::Low,
            3 => Priority::Medium,
            5 => Priority::High,
            _ => Priority::None,
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::None => write!(f, "none"),
            Priority::Low => write!(f, "low"),
            Priority::Medium => write!(f, "medium"),
            Priority::High => write!(f, "high"),
        }
    }
}

impl FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" | "0" => Ok(Priority::None),
            "low" | "1" => Ok(Priority::Low),
            "medium" | "med" | "3" => Ok(Priority::Medium),
            "high" | "5" => Ok(Priority::High),
            _ => Err(format!("Invalid priority: {}", s)),
        }
    }
}

impl Serialize for Priority {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.to_api_value())
    }
}

impl<'de> Deserialize<'de> for Priority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i32::deserialize(deserializer)?;
        Ok(Priority::from_api_value(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_api_values() {
        assert_eq!(Priority::None.to_api_value(), 0);
        assert_eq!(Priority::Low.to_api_value(), 1);
        assert_eq!(Priority::Medium.to_api_value(), 3);
        assert_eq!(Priority::High.to_api_value(), 5);
    }

    #[test]
    fn test_priority_from_api_values() {
        assert_eq!(Priority::from_api_value(0), Priority::None);
        assert_eq!(Priority::from_api_value(1), Priority::Low);
        assert_eq!(Priority::from_api_value(3), Priority::Medium);
        assert_eq!(Priority::from_api_value(5), Priority::High);
        assert_eq!(Priority::from_api_value(99), Priority::None); // Unknown defaults to None
    }

    #[test]
    fn test_priority_from_str() {
        assert_eq!("none".parse::<Priority>().unwrap(), Priority::None);
        assert_eq!("low".parse::<Priority>().unwrap(), Priority::Low);
        assert_eq!("medium".parse::<Priority>().unwrap(), Priority::Medium);
        assert_eq!("high".parse::<Priority>().unwrap(), Priority::High);
        assert_eq!("0".parse::<Priority>().unwrap(), Priority::None);
        assert_eq!("1".parse::<Priority>().unwrap(), Priority::Low);
        assert_eq!("3".parse::<Priority>().unwrap(), Priority::Medium);
        assert_eq!("5".parse::<Priority>().unwrap(), Priority::High);
    }

    #[test]
    fn test_priority_serialization() {
        let priority = Priority::High;
        let json = serde_json::to_string(&priority).unwrap();
        assert_eq!(json, "5");
    }

    #[test]
    fn test_priority_deserialization() {
        let priority: Priority = serde_json::from_str("3").unwrap();
        assert_eq!(priority, Priority::Medium);
    }
}
