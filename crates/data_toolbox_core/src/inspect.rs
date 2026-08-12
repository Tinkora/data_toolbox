use std::{borrow::Cow, collections::BTreeSet};

use crate::{
    CoreError, DelimiterMode, Diagnostic, HeaderMode, InspectOptions, Inspection, MAX_COLUMNS,
    MAX_INPUT_BYTES, MAX_PREVIEW_ROWS, MAX_ROWS, SCHEMA_VERSION, Severity,
};

const DELIMITER_CANDIDATES: [u8; 4] = [b',', b'\t', b';', b'|'];

#[derive(Clone, Debug)]
pub(crate) struct ParsedTable {
    pub(crate) delimiter: u8,
    pub(crate) headers: Vec<String>,
    pub(crate) rows: Vec<Vec<String>>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

pub fn inspect(input: &str, options: &InspectOptions) -> Result<Inspection, CoreError> {
    let table = parse_table(input, options)?;
    let column_count = table
        .rows
        .first()
        .map_or_else(|| table.headers.len(), Vec::len);

    Ok(Inspection {
        schema_version: SCHEMA_VERSION,
        delimiter: char::from(table.delimiter).to_string(),
        headers: table.headers,
        row_count: table.rows.len(),
        column_count,
        preview_rows: table.rows.into_iter().take(MAX_PREVIEW_ROWS).collect(),
        diagnostics: table.diagnostics,
    })
}

pub(crate) fn parse_table(input: &str, options: &InspectOptions) -> Result<ParsedTable, CoreError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(CoreError::InputTooLarge);
    }

    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let normalized = encode_empty_records(input);
    let delimiter = select_delimiter(&normalized, &options.delimiter)?;
    let mut records = parse_records(&normalized, delimiter)?;
    let column_count = records.first().map_or(0, Vec::len);

    if column_count > MAX_COLUMNS {
        return Err(CoreError::ColumnLimitExceeded);
    }

    let headers = match options.headers {
        HeaderMode::Present if records.is_empty() => Vec::new(),
        HeaderMode::Present => records.remove(0),
        HeaderMode::Absent => Vec::new(),
    };

    if records.len() > MAX_ROWS {
        return Err(CoreError::RowLimitExceeded);
    }

    let diagnostics = build_diagnostics(&headers, &records, &options.headers);

    Ok(ParsedTable {
        delimiter,
        headers,
        rows: records,
        diagnostics,
    })
}

fn select_delimiter(input: &str, mode: &DelimiterMode) -> Result<u8, CoreError> {
    match mode {
        DelimiterMode::Comma => Ok(b','),
        DelimiterMode::Tab => Ok(b'\t'),
        DelimiterMode::Semicolon => Ok(b';'),
        DelimiterMode::Pipe => Ok(b'|'),
        DelimiterMode::Auto => detect_delimiter(input),
    }
}

fn detect_delimiter(input: &str) -> Result<u8, CoreError> {
    let mut candidates = Vec::new();
    let mut saw_invalid_csv = false;
    let mut saw_width_mismatch = false;

    for delimiter in DELIMITER_CANDIDATES {
        match parse_records(input, delimiter) {
            Ok(records) => {
                let width = records.first().map_or(0, Vec::len);
                if width > 1 {
                    let score = records.len().saturating_mul(width - 1);
                    candidates.push((delimiter, score));
                }
            }
            Err(CoreError::RowWidthMismatch) => saw_width_mismatch = true,
            Err(CoreError::InvalidCsv) => saw_invalid_csv = true,
            Err(error) => return Err(error),
        }
    }

    let best_score = candidates.iter().map(|(_, score)| *score).max();
    let Some(best_score) = best_score else {
        return if saw_width_mismatch {
            Err(CoreError::RowWidthMismatch)
        } else if saw_invalid_csv {
            Err(CoreError::InvalidCsv)
        } else {
            Err(CoreError::AmbiguousDelimiter)
        };
    };

    let mut best = candidates
        .into_iter()
        .filter(|(_, score)| *score == best_score)
        .map(|(delimiter, _)| delimiter);
    let delimiter = best
        .next()
        .expect("a best score has at least one candidate");

    if best.next().is_some() {
        Err(CoreError::AmbiguousDelimiter)
    } else {
        Ok(delimiter)
    }
}

fn parse_records(input: &str, delimiter: u8) -> Result<Vec<Vec<String>>, CoreError> {
    validate_quotes(input.as_bytes(), delimiter)?;

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(false)
        .trim(csv::Trim::None)
        .from_reader(input.as_bytes());
    let mut records = Vec::new();

    for record in reader.records() {
        match record {
            Ok(record) => records.push(record.iter().map(str::to_owned).collect()),
            Err(error) if matches!(error.kind(), csv::ErrorKind::UnequalLengths { .. }) => {
                return Err(CoreError::RowWidthMismatch);
            }
            Err(_) => return Err(CoreError::InvalidCsv),
        }
    }

    Ok(records)
}

fn encode_empty_records(input: &str) -> Cow<'_, str> {
    let may_contain_empty_record = input.starts_with(['\r', '\n'])
        || input.contains("\n\n")
        || input.contains("\r\r")
        || input.contains("\r\n\r")
        || input.contains("\n\r");
    if !may_contain_empty_record {
        return Cow::Borrowed(input);
    }

    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut inside_quotes = false;
    let mut record_has_content = false;
    let mut index = 0;

    while let Some(&byte) = bytes.get(index) {
        if inside_quotes {
            output.push(byte);
            if byte == b'"' {
                if bytes.get(index + 1) == Some(&b'"') {
                    output.push(b'"');
                    index += 1;
                } else {
                    inside_quotes = false;
                }
            }
            index += 1;
            continue;
        }

        match byte {
            b'"' => {
                inside_quotes = true;
                record_has_content = true;
                output.push(byte);
            }
            b'\r' | b'\n' => {
                if !record_has_content {
                    output.extend_from_slice(b"\"\"");
                }
                output.push(byte);
                if byte == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
                    output.push(b'\n');
                    index += 1;
                }
                record_has_content = false;
            }
            _ => {
                record_has_content = true;
                output.push(byte);
            }
        }
        index += 1;
    }

    Cow::Owned(String::from_utf8(output).expect("input originated as UTF-8"))
}

fn validate_quotes(input: &[u8], delimiter: u8) -> Result<(), CoreError> {
    #[derive(Clone, Copy)]
    enum State {
        FieldStart,
        Unquoted,
        Quoted,
        AfterQuote,
    }

    let mut state = State::FieldStart;
    let mut index = 0;

    while let Some(&byte) = input.get(index) {
        state = match (state, byte) {
            (State::FieldStart, b'"') => State::Quoted,
            (State::FieldStart, value) if value == delimiter => State::FieldStart,
            (State::FieldStart, b'\r' | b'\n') => State::FieldStart,
            (State::FieldStart, _) => State::Unquoted,
            (State::Unquoted, b'"') => return Err(CoreError::InvalidCsv),
            (State::Unquoted, value) if value == delimiter => State::FieldStart,
            (State::Unquoted, b'\r' | b'\n') => State::FieldStart,
            (State::Unquoted, _) => State::Unquoted,
            (State::Quoted, b'"') => State::AfterQuote,
            (State::Quoted, _) => State::Quoted,
            (State::AfterQuote, b'"') => State::Quoted,
            (State::AfterQuote, value) if value == delimiter => State::FieldStart,
            (State::AfterQuote, b'\r' | b'\n') => State::FieldStart,
            (State::AfterQuote, _) => return Err(CoreError::InvalidCsv),
        };

        if byte == b'\r' && input.get(index + 1) == Some(&b'\n') {
            index += 1;
        }
        index += 1;
    }

    if matches!(state, State::Quoted) {
        Err(CoreError::InvalidCsv)
    } else {
        Ok(())
    }
}

fn build_diagnostics(
    headers: &[String],
    rows: &[Vec<String>],
    header_mode: &HeaderMode,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    if matches!(header_mode, HeaderMode::Present) {
        let mut seen = BTreeSet::new();
        for (column_index, header) in headers.iter().enumerate() {
            if header.is_empty() {
                diagnostics.push(warning(
                    "EMPTY_HEADER",
                    "Header is empty.",
                    1,
                    column_index + 1,
                ));
            }
            if !seen.insert(header) {
                diagnostics.push(warning(
                    "DUPLICATE_HEADER",
                    "Header is duplicated.",
                    1,
                    column_index + 1,
                ));
            }
        }

        append_formula_diagnostics(&mut diagnostics, headers, 1);
    }

    let first_data_row = usize::from(matches!(header_mode, HeaderMode::Present)) + 1;
    for (row_index, row) in rows.iter().enumerate() {
        append_formula_diagnostics(&mut diagnostics, row, first_data_row + row_index);
    }

    diagnostics
}

fn append_formula_diagnostics(
    diagnostics: &mut Vec<Diagnostic>,
    row: &[String],
    row_number: usize,
) {
    for (column_index, value) in row.iter().enumerate() {
        if is_formula_like(value) {
            diagnostics.push(warning(
                "FORMULA_LIKE_CELL",
                "Cell may be interpreted as a spreadsheet formula.",
                row_number,
                column_index + 1,
            ));
        }
    }
}

pub(crate) fn is_formula_like(value: &str) -> bool {
    matches!(
        value.trim_start_matches(' ').chars().next(),
        Some('=' | '+' | '-' | '@' | '\t' | '\r')
    )
}

fn warning(code: &str, message: &str, row: usize, column: usize) -> Diagnostic {
    Diagnostic {
        code: code.to_owned(),
        severity: Severity::Warning,
        message: message.to_owned(),
        row: Some(row),
        column: Some(column),
    }
}
