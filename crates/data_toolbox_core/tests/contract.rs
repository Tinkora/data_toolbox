use data_toolbox_core::{
    Conversion, ConvertOptions, CoreError, DelimiterMode, FormulaPolicy, HeaderMode,
    InspectOptions, Inspection, MAX_COLUMNS, MAX_INPUT_BYTES, MAX_PREVIEW_ROWS, MAX_ROWS,
    OutputFormat, SCHEMA_VERSION,
};

#[test]
fn inspection_uses_versioned_snake_case_contract() {
    let inspection = Inspection {
        schema_version: SCHEMA_VERSION,
        delimiter: String::new(),
        headers: Vec::new(),
        row_count: 0,
        column_count: 0,
        preview_rows: Vec::new(),
        diagnostics: Vec::new(),
    };

    let value = serde_json::to_value(inspection).expect("inspection serializes");

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["row_count"], 0);
    assert!(value.get("schemaVersion").is_none());
}

#[test]
fn resource_limits_are_stable() {
    assert_eq!(MAX_INPUT_BYTES, 10 * 1024 * 1024);
    assert_eq!(MAX_ROWS, 200_000);
    assert_eq!(MAX_COLUMNS, 1_024);
    assert_eq!(MAX_PREVIEW_ROWS, 100);
}

#[test]
fn adapter_models_serialize_with_snake_case_values() {
    let inspect_options = InspectOptions {
        delimiter: DelimiterMode::Auto,
        headers: HeaderMode::Present,
    };
    let convert_options = ConvertOptions {
        delimiter: DelimiterMode::Tab,
        headers: HeaderMode::Absent,
        output: OutputFormat::Json,
        formula_policy: FormulaPolicy::EscapeForSpreadsheet,
    };
    let conversion = Conversion {
        schema_version: SCHEMA_VERSION,
        output: OutputFormat::Json,
        content: "[]".to_owned(),
        diagnostics: Vec::new(),
    };

    assert_eq!(
        serde_json::to_value(inspect_options).expect("inspect options serialize"),
        serde_json::json!({ "delimiter": "auto", "headers": "present" })
    );
    assert_eq!(
        serde_json::to_value(convert_options).expect("convert options serialize"),
        serde_json::json!({
            "delimiter": "tab",
            "headers": "absent",
            "output": "json",
            "formula_policy": "escape_for_spreadsheet"
        })
    );
    assert_eq!(
        serde_json::to_value(conversion).expect("conversion serializes"),
        serde_json::json!({
            "schema_version": 1,
            "output": "json",
            "content": "[]",
            "diagnostics": []
        })
    );
}

#[test]
fn fatal_errors_have_stable_codes_and_serializable_envelopes() {
    let errors = [
        (CoreError::InputTooLarge, "INPUT_TOO_LARGE"),
        (CoreError::RowLimitExceeded, "ROW_LIMIT_EXCEEDED"),
        (CoreError::ColumnLimitExceeded, "COLUMN_LIMIT_EXCEEDED"),
        (CoreError::InvalidCsv, "INVALID_CSV"),
        (CoreError::AmbiguousDelimiter, "AMBIGUOUS_DELIMITER"),
        (CoreError::RowWidthMismatch, "ROW_WIDTH_MISMATCH"),
        (
            CoreError::JsonExportRequiresUniqueHeaders,
            "JSON_EXPORT_REQUIRES_UNIQUE_HEADERS",
        ),
        (CoreError::InvalidOptions, "INVALID_OPTIONS"),
    ];

    for (error, expected_code) in errors {
        let envelope = error.to_envelope();
        let value = serde_json::to_value(&envelope).expect("error envelope serializes");

        assert_eq!(error.code(), expected_code);
        assert_eq!(value["code"], expected_code);
        assert_eq!(value["message"], error.to_string());
        assert!(!error.to_string().is_empty());
    }
}
