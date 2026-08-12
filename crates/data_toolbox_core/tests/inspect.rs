use data_toolbox_core::{
    CoreError, DelimiterMode, HeaderMode, InspectOptions, MAX_COLUMNS, MAX_INPUT_BYTES,
    MAX_PREVIEW_ROWS, MAX_ROWS, inspect,
};

fn present_headers() -> InspectOptions {
    InspectOptions {
        delimiter: DelimiterMode::Comma,
        headers: HeaderMode::Present,
    }
}

#[test]
fn quoted_newline_and_spaces_are_preserved() {
    let input = "name,note\r\nAlice,\"  first\nsecond  \"\r\n";

    let result = inspect(input, &present_headers()).expect("valid CSV");

    assert_eq!(result.preview_rows[0][1], "  first\nsecond  ");
}

#[test]
fn empty_records_are_not_silently_discarded() {
    let result = inspect("name\n\nAlice\n", &present_headers()).expect("valid CSV");

    assert_eq!(result.row_count, 2);
    assert_eq!(result.preview_rows, [[""], ["Alice"]]);
}

#[test]
fn empty_records_do_not_trigger_implicit_column_padding() {
    let error = inspect("name,role\n\nAlice,owner\n", &present_headers())
        .expect_err("empty record has the wrong width");

    assert_eq!(error, CoreError::RowWidthMismatch);
}

#[test]
fn blank_lines_inside_quoted_fields_are_preserved() {
    let input = "name,note\nAlice,\"first\n\nthird\"\n";

    let result = inspect(input, &present_headers()).expect("valid quoted field");

    assert_eq!(result.preview_rows, [["Alice", "first\n\nthird"]]);
}

#[test]
fn jagged_record_is_rejected_without_padding_or_truncation() {
    let error = inspect("a,b\n1\n", &present_headers()).expect_err("jagged input");

    assert_eq!(error.code(), "ROW_WIDTH_MISMATCH");
}

#[test]
fn duplicate_headers_and_formula_cells_are_reported_in_order() {
    let result = inspect("name,name\nAlice,=1+1\n", &present_headers()).expect("valid CSV");
    let codes: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect();

    assert_eq!(codes, ["DUPLICATE_HEADER", "FORMULA_LIKE_CELL"]);
}

#[test]
fn utf8_bom_is_ignored_without_changing_the_first_header() {
    let result = inspect("\u{feff}name,role\nAlice,owner\n", &present_headers())
        .expect("valid BOM-prefixed CSV");

    assert_eq!(result.headers, ["name", "role"]);
}

#[test]
fn every_explicit_delimiter_uses_the_same_strict_parser() {
    let cases = [
        (DelimiterMode::Comma, "name,role\nAlice,owner\n", ","),
        (DelimiterMode::Tab, "name\trole\nAlice\towner\n", "\t"),
        (DelimiterMode::Semicolon, "name;role\nAlice;owner\n", ";"),
        (DelimiterMode::Pipe, "name|role\nAlice|owner\n", "|"),
    ];

    for (delimiter, input, expected) in cases {
        let options = InspectOptions {
            delimiter,
            headers: HeaderMode::Present,
        };
        let result = inspect(input, &options).expect("valid delimited input");

        assert_eq!(result.delimiter, expected);
        assert_eq!(result.headers, ["name", "role"]);
        assert_eq!(result.preview_rows, [["Alice", "owner"]]);
    }
}

#[test]
fn absent_header_mode_keeps_the_first_record_as_data() {
    let options = InspectOptions {
        delimiter: DelimiterMode::Comma,
        headers: HeaderMode::Absent,
    };

    let result = inspect("Alice,owner\nBob,reviewer\n", &options).expect("valid CSV");

    assert!(result.headers.is_empty());
    assert_eq!(result.row_count, 2);
    assert_eq!(
        result.preview_rows,
        [["Alice", "owner"], ["Bob", "reviewer"]]
    );
}

#[test]
fn auto_detection_rejects_tied_plausible_dialects() {
    let options = InspectOptions {
        delimiter: DelimiterMode::Auto,
        headers: HeaderMode::Present,
    };

    let error =
        inspect("name,role;team\nAlice,owner;core\n", &options).expect_err("ambiguous dialect");

    assert_eq!(error, CoreError::AmbiguousDelimiter);
}

#[test]
fn auto_detection_ignores_delimiters_inside_quoted_fields() {
    let options = InspectOptions {
        delimiter: DelimiterMode::Auto,
        headers: HeaderMode::Present,
    };

    let result = inspect("name;note\nAlice;\"uses, commas | safely\"\n", &options)
        .expect("unambiguous semicolon input");

    assert_eq!(result.delimiter, ";");
    assert_eq!(result.preview_rows, [["Alice", "uses, commas | safely"]]);
}

#[test]
fn malformed_quotes_are_rejected() {
    let error = inspect("name,note\nAlice,\"unterminated\n", &present_headers())
        .expect_err("malformed CSV");

    assert_eq!(error.code(), "INVALID_CSV");
}

#[test]
fn empty_headers_are_reported_without_rewriting_them() {
    let result = inspect("name,\nAlice,owner\n", &present_headers()).expect("valid CSV");

    assert_eq!(result.headers, ["name", ""]);
    assert_eq!(result.diagnostics[0].code, "EMPTY_HEADER");
    assert_eq!(result.diagnostics[0].column, Some(2));
}

#[test]
fn input_over_the_byte_limit_is_rejected_before_parsing() {
    let input = "a".repeat(MAX_INPUT_BYTES + 1);

    let error = inspect(&input, &present_headers()).expect_err("input over limit");

    assert_eq!(error, CoreError::InputTooLarge);
}

#[test]
fn row_limit_is_enforced_without_truncating_the_extra_record() {
    let mut input = String::from("name\n");
    input.push_str(&"Alice\n".repeat(MAX_ROWS + 1));

    let error = inspect(&input, &present_headers()).expect_err("input over row limit");

    assert_eq!(error, CoreError::RowLimitExceeded);
}

#[test]
fn column_limit_is_enforced_without_dropping_extra_columns() {
    let input = format!(
        "{}\n{}\n",
        "h,".repeat(MAX_COLUMNS),
        "v,".repeat(MAX_COLUMNS)
    );

    let error = inspect(&input, &present_headers()).expect_err("input over column limit");

    assert_eq!(error, CoreError::ColumnLimitExceeded);
}

#[test]
fn preview_is_bounded_without_changing_the_row_count() {
    let input = format!("name\n{}", "Alice\n".repeat(MAX_PREVIEW_ROWS + 1));

    let result = inspect(&input, &present_headers()).expect("valid CSV");

    assert_eq!(result.row_count, MAX_PREVIEW_ROWS + 1);
    assert_eq!(result.preview_rows.len(), MAX_PREVIEW_ROWS);
}
