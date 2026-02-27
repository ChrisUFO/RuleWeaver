use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::parse_error::ParseEnumError;

/// Type of reconciliation operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconcileOperation {
    Create,
    Update,
    Remove,
    Check,
}

impl ReconcileOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Remove => "remove",
            Self::Check => "check",
        }
    }
}

impl FromStr for ReconcileOperation {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" => Ok(Self::Create),
            "update" => Ok(Self::Update),
            "remove" => Ok(Self::Remove),
            "check" => Ok(Self::Check),
            _ => Err(ParseEnumError),
        }
    }
}

/// Result type for a single reconciliation action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconcileResultType {
    Success,
    Failed,
    Skipped,
}

impl ReconcileResultType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

impl FromStr for ReconcileResultType {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "success" => Ok(Self::Success),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            _ => Err(ParseEnumError),
        }
    }
}
