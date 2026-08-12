mod error;
mod inspect;
mod model;

pub use error::{CoreError, ErrorEnvelope};
pub use inspect::inspect;
pub use model::{
    Conversion, ConvertOptions, DelimiterMode, Diagnostic, FormulaPolicy, HeaderMode,
    InspectOptions, Inspection, OutputFormat, Severity,
};

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_INPUT_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_ROWS: usize = 200_000;
pub const MAX_COLUMNS: usize = 1_024;
pub const MAX_PREVIEW_ROWS: usize = 100;
