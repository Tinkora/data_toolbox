use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{self, Read, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use data_toolbox_core::{
    ConvertOptions, CoreError, DelimiterMode, FormulaPolicy, HeaderMode, InspectOptions,
    MAX_INPUT_BYTES, OutputFormat, convert, inspect,
};

enum Command {
    Inspect {
        options: InspectOptions,
        input: PathBuf,
    },
    Convert {
        options: ConvertOptions,
        input: PathBuf,
    },
}

enum AppError {
    Core(CoreError),
    InvalidOptions(&'static str),
    InvalidUtf8,
    Io,
}

impl AppError {
    const fn code(&self) -> &'static str {
        match self {
            Self::Core(error) => error.code(),
            Self::InvalidOptions(_) => "INVALID_OPTIONS",
            Self::InvalidUtf8 => "INVALID_UTF8",
            Self::Io => "IO_ERROR",
        }
    }

    const fn message(&self) -> &str {
        match self {
            Self::Core(CoreError::InputTooLarge) => "Input exceeds the 10 MiB limit.",
            Self::Core(CoreError::RowLimitExceeded) => "Input exceeds the 200,000-row limit.",
            Self::Core(CoreError::ColumnLimitExceeded) => "Input exceeds the 1,024-column limit.",
            Self::Core(CoreError::InvalidCsv) => "Input is not valid CSV.",
            Self::Core(CoreError::AmbiguousDelimiter) => {
                "Delimiter could not be selected unambiguously."
            }
            Self::Core(CoreError::RowWidthMismatch) => "Rows contain inconsistent column counts.",
            Self::Core(CoreError::JsonExportRequiresUniqueHeaders) => {
                "JSON export requires non-empty, unique headers."
            }
            Self::Core(CoreError::InvalidOptions) => "Options are invalid.",
            Self::InvalidOptions(message) => message,
            Self::InvalidUtf8 => "Input must be valid UTF-8 text.",
            Self::Io => "Input or output could not be read or written.",
        }
    }

    const fn exit_code(&self) -> u8 {
        if matches!(self, Self::InvalidOptions(_)) {
            2
        } else {
            1
        }
    }
}

impl From<CoreError> for AppError {
    fn from(error: CoreError) -> Self {
        Self::Core(error)
    }
}

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            write_error(&error);
            ExitCode::from(error.exit_code())
        }
    }
}

fn run(args: impl Iterator<Item = OsString>) -> Result<(), AppError> {
    match parse_args(args)? {
        Command::Inspect { options, input } => {
            let input = read_input(&input)?;
            let result = inspect(&input, &options)?;
            serde_json::to_writer(io::stdout().lock(), &result).map_err(|_| AppError::Io)
        }
        Command::Convert { options, input } => {
            let input = read_input(&input)?;
            let result = convert(&input, &options)?;
            io::stdout()
                .lock()
                .write_all(result.content.as_bytes())
                .map_err(|_| AppError::Io)
        }
    }
}

fn parse_args(mut args: impl Iterator<Item = OsString>) -> Result<Command, AppError> {
    let command = args.next().ok_or(AppError::InvalidOptions(
        "Expected the inspect or convert subcommand.",
    ))?;
    let is_convert = match command.to_str() {
        Some("inspect") => false,
        Some("convert") => true,
        _ => {
            return Err(AppError::InvalidOptions(
                "Expected the inspect or convert subcommand.",
            ));
        }
    };

    let mut delimiter = DelimiterMode::Auto;
    let mut headers = HeaderMode::Present;
    let mut output = None;
    let mut formula_policy = FormulaPolicy::Preserve;
    let mut input = None;
    let mut saw_delimiter = false;
    let mut saw_headers = false;
    let mut saw_formula_policy = false;

    while let Some(argument) = args.next() {
        match argument.to_str() {
            Some("--delimiter") => {
                reject_repeated(&mut saw_delimiter, "--delimiter may be provided only once.")?;
                delimiter =
                    parse_delimiter(next_value(&mut args, "--delimiter requires a value")?)?;
            }
            Some("--headers") => {
                reject_repeated(&mut saw_headers, "--headers may be provided only once.")?;
                headers = parse_headers(next_value(&mut args, "--headers requires a value")?)?;
            }
            Some("--to") if is_convert => {
                if output.is_some() {
                    return Err(AppError::InvalidOptions("--to may be provided only once."));
                }
                output = Some(parse_output(next_value(
                    &mut args,
                    "--to requires a value",
                )?)?);
            }
            Some("--formula-policy") if is_convert => {
                reject_repeated(
                    &mut saw_formula_policy,
                    "--formula-policy may be provided only once.",
                )?;
                formula_policy = parse_formula_policy(next_value(
                    &mut args,
                    "--formula-policy requires a value",
                )?)?;
            }
            Some(value) if !value.starts_with('-') || value == "-" => {
                if input.is_some() {
                    return Err(AppError::InvalidOptions(
                        "Only one input file may be provided.",
                    ));
                }
                input = Some(PathBuf::from(argument));
            }
            _ => return Err(AppError::InvalidOptions("Unknown option.")),
        }
    }

    let input = input.unwrap_or_else(|| PathBuf::from("-"));
    if is_convert {
        Ok(Command::Convert {
            options: ConvertOptions {
                delimiter,
                headers,
                output: output.ok_or(AppError::InvalidOptions(
                    "The convert subcommand requires --to.",
                ))?,
                formula_policy,
            },
            input,
        })
    } else {
        Ok(Command::Inspect {
            options: InspectOptions { delimiter, headers },
            input,
        })
    }
}

fn reject_repeated(seen: &mut bool, message: &'static str) -> Result<(), AppError> {
    if *seen {
        Err(AppError::InvalidOptions(message))
    } else {
        *seen = true;
        Ok(())
    }
}

fn next_value(
    args: &mut impl Iterator<Item = OsString>,
    missing_message: &'static str,
) -> Result<OsString, AppError> {
    args.next().ok_or(AppError::InvalidOptions(missing_message))
}

fn parse_delimiter(value: OsString) -> Result<DelimiterMode, AppError> {
    match value.to_str() {
        Some("auto") => Ok(DelimiterMode::Auto),
        Some("comma") => Ok(DelimiterMode::Comma),
        Some("tab") => Ok(DelimiterMode::Tab),
        Some("semicolon") => Ok(DelimiterMode::Semicolon),
        Some("pipe") => Ok(DelimiterMode::Pipe),
        _ => Err(AppError::InvalidOptions("Unsupported delimiter.")),
    }
}

fn parse_headers(value: OsString) -> Result<HeaderMode, AppError> {
    match value.to_str() {
        Some("present") => Ok(HeaderMode::Present),
        Some("absent") => Ok(HeaderMode::Absent),
        _ => Err(AppError::InvalidOptions("Unsupported header mode.")),
    }
}

fn parse_output(value: OsString) -> Result<OutputFormat, AppError> {
    match value.to_str() {
        Some("csv") => Ok(OutputFormat::Csv),
        Some("tsv") => Ok(OutputFormat::Tsv),
        Some("json") => Ok(OutputFormat::Json),
        _ => Err(AppError::InvalidOptions("Unsupported output format.")),
    }
}

fn parse_formula_policy(value: OsString) -> Result<FormulaPolicy, AppError> {
    match value.to_str() {
        Some("preserve") => Ok(FormulaPolicy::Preserve),
        Some("escape_for_spreadsheet") => Ok(FormulaPolicy::EscapeForSpreadsheet),
        _ => Err(AppError::InvalidOptions("Unsupported formula policy.")),
    }
}

fn read_input(path: &Path) -> Result<String, AppError> {
    let bytes = if path == OsStr::new("-") {
        read_bounded(io::stdin().lock())?
    } else {
        let metadata = path.metadata().map_err(|_| AppError::Io)?;
        if metadata.len() > MAX_INPUT_BYTES as u64 {
            return Err(AppError::Core(CoreError::InputTooLarge));
        }
        read_bounded(File::open(path).map_err(|_| AppError::Io)?)?
    };

    String::from_utf8(bytes).map_err(|_| AppError::InvalidUtf8)
}

fn read_bounded(reader: impl Read) -> Result<Vec<u8>, AppError> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AppError::Io)?;
    if bytes.len() > MAX_INPUT_BYTES {
        Err(AppError::Core(CoreError::InputTooLarge))
    } else {
        Ok(bytes)
    }
}

fn write_error(error: &AppError) {
    let envelope = serde_json::json!({
        "code": error.code(),
        "message": error.message(),
    });
    let mut stderr = io::stderr().lock();
    let _ = serde_json::to_writer(&mut stderr, &envelope);
    let _ = stderr.write_all(b"\n");
}
