use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("Input exceeds the 10 MiB limit.")]
    InputTooLarge,
    #[error("Input exceeds the 200,000-row limit.")]
    RowLimitExceeded,
    #[error("Input exceeds the 1,024-column limit.")]
    ColumnLimitExceeded,
    #[error("Input is not valid CSV.")]
    InvalidCsv,
    #[error("Delimiter could not be selected unambiguously.")]
    AmbiguousDelimiter,
    #[error("Rows contain inconsistent column counts.")]
    RowWidthMismatch,
    #[error("JSON export requires non-empty, unique headers.")]
    JsonExportRequiresUniqueHeaders,
    #[error("Options are invalid.")]
    InvalidOptions,
}

impl CoreError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InputTooLarge => "INPUT_TOO_LARGE",
            Self::RowLimitExceeded => "ROW_LIMIT_EXCEEDED",
            Self::ColumnLimitExceeded => "COLUMN_LIMIT_EXCEEDED",
            Self::InvalidCsv => "INVALID_CSV",
            Self::AmbiguousDelimiter => "AMBIGUOUS_DELIMITER",
            Self::RowWidthMismatch => "ROW_WIDTH_MISMATCH",
            Self::JsonExportRequiresUniqueHeaders => "JSON_EXPORT_REQUIRES_UNIQUE_HEADERS",
            Self::InvalidOptions => "INVALID_OPTIONS",
        }
    }

    pub fn to_envelope(&self) -> ErrorEnvelope {
        ErrorEnvelope {
            code: self.code().to_owned(),
            message: self.to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ErrorEnvelope {
    pub code: String,
    pub message: String,
}
