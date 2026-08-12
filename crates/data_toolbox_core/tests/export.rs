use data_toolbox_core::{
    ConvertOptions, CoreError, DelimiterMode, FormulaPolicy, HeaderMode, OutputFormat, convert,
};

fn csv_options(formula_policy: FormulaPolicy) -> ConvertOptions {
    ConvertOptions {
        delimiter: DelimiterMode::Comma,
        headers: HeaderMode::Present,
        output: OutputFormat::Csv,
        formula_policy,
    }
}

fn json_options() -> ConvertOptions {
    ConvertOptions {
        output: OutputFormat::Json,
        ..csv_options(FormulaPolicy::Preserve)
    }
}

#[test]
fn preserve_policy_reports_but_does_not_change_formula_text() {
    let output =
        convert("value\n=1+1\n", &csv_options(FormulaPolicy::Preserve)).expect("valid CSV");

    assert_eq!(output.content, "value\r\n=1+1\r\n");
    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(output.diagnostics[0].code, "FORMULA_LIKE_CELL");
    assert_eq!(output.diagnostics[0].row, Some(2));
    assert_eq!(output.diagnostics[0].column, Some(1));
}

#[test]
fn explicit_spreadsheet_policy_prefixes_formula_like_cells_once() {
    let output = convert(
        "value,escaped\n  @SUM(A1),'=literal\n",
        &csv_options(FormulaPolicy::EscapeForSpreadsheet),
    )
    .expect("valid CSV");

    assert_eq!(output.content, "value,escaped\r\n'  @SUM(A1),'=literal\r\n");
    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(output.diagnostics[0].code, "FORMULA_LIKE_CELL");
}

#[test]
fn spreadsheet_policy_covers_every_declared_formula_prefix() {
    let cases = ["=1+1", "+1", "-1", "@SUM(A1)", "\tvalue", "\rvalue"];

    for value in cases {
        let input = format!("value\n\"{value}\"\n");
        let output =
            convert(&input, &csv_options(FormulaPolicy::EscapeForSpreadsheet)).expect("valid CSV");

        assert!(output.content.contains(&format!("'{value}")));
        assert_eq!(output.diagnostics[0].code, "FORMULA_LIKE_CELL");
    }
}

#[test]
fn csv_export_canonically_quotes_commas_quotes_crlf_and_empty_final_fields() {
    let input = "name,note,last\r\nAlice,\"line 1\r\nline \"\"2\"\", comma\",\r\n";

    let output = convert(input, &csv_options(FormulaPolicy::Preserve)).expect("valid CSV");

    assert_eq!(
        output.content,
        "name,note,last\r\nAlice,\"line 1\r\nline \"\"2\"\", comma\",\r\n"
    );
}

#[test]
fn tsv_export_quotes_embedded_tabs() {
    let options = ConvertOptions {
        output: OutputFormat::Tsv,
        ..csv_options(FormulaPolicy::Preserve)
    };

    let output = convert("name,note\nAlice,a\tb\n", &options).expect("valid CSV");

    assert_eq!(output.content, "name\tnote\r\nAlice\t\"a\tb\"\r\n");
}

#[test]
fn csv_with_absent_headers_keeps_the_first_record() {
    let options = ConvertOptions {
        headers: HeaderMode::Absent,
        ..csv_options(FormulaPolicy::Preserve)
    };

    let output = convert("Alice,owner\nBob,reviewer\n", &options).expect("valid CSV");

    assert_eq!(output.content, "Alice,owner\r\nBob,reviewer\r\n");
}

#[test]
fn json_uses_present_headers_in_input_order_deterministically() {
    let first = convert("z,a\n1,2\n3,4\n", &json_options()).expect("valid CSV");
    let second = convert("z,a\n1,2\n3,4\n", &json_options()).expect("valid CSV");

    assert_eq!(first.content, r#"[{"z":"1","a":"2"},{"z":"3","a":"4"}]"#);
    assert_eq!(second.content, first.content);
}

#[test]
fn json_with_absent_headers_exports_rows_as_arrays() {
    let options = ConvertOptions {
        headers: HeaderMode::Absent,
        ..json_options()
    };

    let output = convert("Alice,owner\nBob,reviewer\n", &options).expect("valid CSV");

    assert_eq!(output.content, r#"[["Alice","owner"],["Bob","reviewer"]]"#);
}

#[test]
fn json_always_preserves_formula_like_values() {
    let options = ConvertOptions {
        formula_policy: FormulaPolicy::EscapeForSpreadsheet,
        ..json_options()
    };

    let output = convert("value\n=1+1\n", &options).expect("valid CSV");

    assert_eq!(output.content, r#"[{"value":"=1+1"}]"#);
    assert_eq!(output.diagnostics[0].code, "FORMULA_LIKE_CELL");
}

#[test]
fn json_refuses_duplicate_present_headers() {
    let error = convert("x,x\n1,2\n", &json_options()).expect_err("duplicate headers");

    assert_eq!(error, CoreError::JsonExportRequiresUniqueHeaders);
}

#[test]
fn json_refuses_empty_present_headers() {
    let error = convert("x,\n1,2\n", &json_options()).expect_err("empty header");

    assert_eq!(error, CoreError::JsonExportRequiresUniqueHeaders);
}
