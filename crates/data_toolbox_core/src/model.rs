use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DelimiterMode {
    Auto,
    Comma,
    Tab,
    Semicolon,
    Pipe,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HeaderMode {
    Present,
    Absent,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub row: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Inspection {
    pub schema_version: u32,
    pub delimiter: String,
    pub headers: Vec<String>,
    pub row_count: usize,
    pub column_count: usize,
    pub preview_rows: Vec<Vec<String>>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct InspectOptions {
    pub delimiter: DelimiterMode,
    pub headers: HeaderMode,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Csv,
    Tsv,
    Json,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FormulaPolicy {
    Preserve,
    EscapeForSpreadsheet,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ConvertOptions {
    pub delimiter: DelimiterMode,
    pub headers: HeaderMode,
    pub output: OutputFormat,
    pub formula_policy: FormulaPolicy,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Conversion {
    pub schema_version: u32,
    pub output: OutputFormat,
    pub content: String,
    pub diagnostics: Vec<Diagnostic>,
}
