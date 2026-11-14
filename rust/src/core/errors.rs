use std::fmt::{Display, Formatter};

use super::ErrorCode;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HarnessError {
    pub code: ErrorCode,
    pub message: &'static str,
}

impl HarnessError {
    pub const fn new(code: ErrorCode, message: &'static str) -> Self {
        Self { code, message }
    }
}

impl Display for HarnessError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)
    }
}

impl std::error::Error for HarnessError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryError {
    pub kind: &'static str,
    pub id: String,
}

impl Display for RegistryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} already registered: {}", self.kind, self.id)
    }
}

impl std::error::Error for RegistryError {}
