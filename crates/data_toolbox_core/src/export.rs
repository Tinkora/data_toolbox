use std::{borrow::Cow, collections::BTreeSet};

use serde_json::{Map, Value};

use crate::{
    Conversion, ConvertOptions, CoreError, FormulaPolicy, HeaderMode, InspectOptions, OutputFormat,
    SCHEMA_VERSION,
    inspect::{ParsedTable, is_formula_like, parse_table},
};

pub fn convert(input: &str, options: &ConvertOptions) -> Result<Conversion, CoreError> {
    let parsed = parse_table(
        input,
        &InspectOptions {
            delimiter: options.delimiter.clone(),
            headers: options.headers.clone(),
        },
    )?;

    let content = match options.output {
        OutputFormat::Csv => write_delimited(&parsed, b',', &options.formula_policy)?,
        OutputFormat::Tsv => write_delimited(&parsed, b'\t', &options.formula_policy)?,
        OutputFormat::Json => write_json_rows(&parsed, &options.headers)?,
    };

    Ok(Conversion {
        schema_version: SCHEMA_VERSION,
        output: options.output.clone(),
        content,
        diagnostics: parsed.diagnostics,
    })
}

fn write_delimited(
    parsed: &ParsedTable,
    delimiter: u8,
    formula_policy: &FormulaPolicy,
) -> Result<String, CoreError> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .terminator(csv::Terminator::CRLF)
        .from_writer(Vec::new());

    if !parsed.headers.is_empty() {
        write_record(&mut writer, &parsed.headers, formula_policy)?;
    }
    for row in &parsed.rows {
        write_record(&mut writer, row, formula_policy)?;
    }

    let bytes = writer.into_inner().map_err(|_| CoreError::InvalidOptions)?;
    String::from_utf8(bytes).map_err(|_| CoreError::InvalidOptions)
}

fn write_record(
    writer: &mut csv::Writer<Vec<u8>>,
    record: &[String],
    formula_policy: &FormulaPolicy,
) -> Result<(), CoreError> {
    let fields: Vec<_> = record
        .iter()
        .map(|value| protect_formula(value, formula_policy))
        .collect();
    writer
        .write_record(fields.iter().map(|field| field.as_bytes()))
        .map_err(|_| CoreError::InvalidOptions)
}

fn protect_formula<'a>(value: &'a str, policy: &FormulaPolicy) -> Cow<'a, str> {
    if matches!(policy, FormulaPolicy::EscapeForSpreadsheet) && is_formula_like(value) {
        Cow::Owned(format!("'{value}"))
    } else {
        Cow::Borrowed(value)
    }
}

fn write_json_rows(parsed: &ParsedTable, header_mode: &HeaderMode) -> Result<String, CoreError> {
    let value = match header_mode {
        HeaderMode::Present => {
            validate_json_headers(&parsed.headers)?;
            Value::Array(
                parsed
                    .rows
                    .iter()
                    .map(|row| {
                        let mut object = Map::new();
                        for (header, field) in parsed.headers.iter().zip(row) {
                            object.insert(header.clone(), Value::String(field.clone()));
                        }
                        Value::Object(object)
                    })
                    .collect(),
            )
        }
        HeaderMode::Absent => Value::Array(
            parsed
                .rows
                .iter()
                .map(|row| Value::Array(row.iter().cloned().map(Value::String).collect::<Vec<_>>()))
                .collect(),
        ),
    };

    serde_json::to_string(&value).map_err(|_| CoreError::InvalidOptions)
}

fn validate_json_headers(headers: &[String]) -> Result<(), CoreError> {
    let mut seen = BTreeSet::new();
    if headers
        .iter()
        .any(|header| header.is_empty() || !seen.insert(header))
    {
        Err(CoreError::JsonExportRequiresUniqueHeaders)
    } else {
        Ok(())
    }
}
