# Design: Data Toolbox CSV Inspector

## Objective

Build a local CSV/TSV inspector for developers and AI-agent operators who need
to examine a sensitive, one-off delimited file before passing it to a
spreadsheet, script, or tool. It must make structural uncertainty and
spreadsheet-formula risk visible instead of silently changing user data.

Success means a user can paste or open a CSV/TSV file up to 10 MiB, see a
bounded structural report, and export deterministic CSV, TSV, or JSON with an
explicit data-preservation policy. The same behavior is available through a
native CLI with JSON diagnostics. This is **machine-readable**, not
Agent-callable or an MCP server.

### Assumptions

1. The first release targets current desktop browsers and native macOS, Linux,
   and Windows command lines, not mobile or hosted use.
2. The browser interface and CLI must call the same Rust core.
3. Input is untrusted text and must never leave process memory.
4. CSV dialect detection is limited to comma, tab, semicolon, and pipe. A
   caller can always select one explicitly.
5. The standing Tinkora delivery authorization approves this bounded design;
   any change that adds a network boundary, persistence, a new external data
   source, or an MCP transport requires a new decision.

## Commands

```text
# Rust quality
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p data_toolbox_web --target wasm32-unknown-unknown --locked

# Browser quality
cd crates/data_toolbox_web
npm ci
npm run build:wasm
npm run test:wasm-smoke

# Documentation quality
npx --yes markdownlint-cli2@0.23.2 "**/*.md"
ruby scripts/test_check_docs.rb
ruby scripts/check_docs.rb
ruby scripts/test_check_workflow_contracts.rb
ruby scripts/check_workflow_contracts.rb
```

## Project Structure

```text
crates/data_toolbox_core/       strict parser, diagnostics, exporters, tests
crates/data_toolbox_cli/        stdin/file CLI and JSON error envelope
crates/data_toolbox_web/        thin WASM boundary and local browser tool
docs/decisions/                 product and architecture decisions
docs/superpowers/specs/         accepted design
docs/superpowers/plans/         executable implementation plan
scripts/                        offline documentation and workflow contracts
```

## Core Contract

The core uses data structures with stable `snake_case` serialized field names.
All successful structured results contain `schema_version: 1`.

```rust
pub fn inspect(input: &str, options: &InspectOptions) -> Result<Inspection, CoreError>;
pub fn convert(input: &str, options: &ConvertOptions) -> Result<Conversion, CoreError>;
```

`InspectOptions` requires `delimiter` (`auto`, `comma`, `tab`, `semicolon`,
or `pipe`) and `headers` (`present` or `absent`). It does not guess whether a
first row is a header. `InspectOptions` also applies fixed limits of 10 MiB,
200,000 records, 1,024 columns, and 100 preview rows.

`Inspection` contains the selected delimiter, header list, row and column
counts, a maximum-100-row preview, and ordered diagnostics. No diagnostic
changes a parsed cell. `Conversion` contains an output format, content, and
the diagnostics that were considered.

Diagnostics use stable uppercase codes:

- `INPUT_TOO_LARGE`, `ROW_LIMIT_EXCEEDED`, `COLUMN_LIMIT_EXCEEDED`
- `INVALID_CSV`, `AMBIGUOUS_DELIMITER`, `ROW_WIDTH_MISMATCH`
- `EMPTY_HEADER`, `DUPLICATE_HEADER`, `FORMULA_LIKE_CELL`
- `JSON_EXPORT_REQUIRES_UNIQUE_HEADERS`

Formula-like cells are values whose first non-space character is `=`, `+`,
`-`, `@`, tab, or carriage return. The default `preserve` policy reports them
without altering the export. The explicit `escape_for_spreadsheet` policy
prefixes such CSV/TSV cells with a single apostrophe. JSON is always preserved
verbatim.

## CLI Contract

```text
data-toolbox inspect [--delimiter auto|comma|tab|semicolon|pipe] \
  [--headers present|absent] [FILE|-]

data-toolbox convert --to csv|tsv|json \
  [--delimiter auto|comma|tab|semicolon|pipe] \
  [--headers present|absent] \
  [--formula-policy preserve|escape_for_spreadsheet] [FILE|-]
```

`inspect` writes one JSON object to stdout. `convert` writes only converted
content to stdout. Both commands write one JSON error object to stderr and
exit nonzero on failure. The CLI reads an explicit file or stdin, never scans
the filesystem, and never opens a network connection.

## Browser Contract

The browser tool accepts a file or pasted text, has explicit delimiter and
header controls, and renders all untrusted cell content through DOM text APIs.
It displays the first 100 rows, diagnostics, and export controls. It has no
remote fonts, analytics, local storage, JavaScript parser fallback, or dynamic
HTML string interpolation of input data.

## Code Style

Public APIs return structured errors and never turn malformed input into
plausible data:

```rust
if record.len() != expected_columns {
    return Err(CoreError::row_width_mismatch(row_number, expected_columns, record.len()));
}
```

Use English comments only for invariants and trust boundaries. Keep parsing,
diagnostics, export, CLI, and browser rendering in separate modules. Do not
use `HashMap` where serialized key order or duplicate-key detection matters.

## Testing Strategy

Small Rust tests lead implementation. Every feature follows RED, GREEN,
REFACTOR: add a failing test, run it to prove the failure, implement the
minimum behavior, and rerun the focused test before the full suite.

Core fixtures cover RFC 4180 quoted commas/newlines, CRLF, BOM, explicit
delimiter choice, ambiguous detection, malformed quotes, jagged records,
duplicate and empty headers, formula-like values, all limits, deterministic
CSV/TSV output, and JSON rejection for non-unique headers. CLI tests exercise
stdin, file input, exit codes, stdout, and JSON stderr. Browser smoke tests
exercise the real WASM artifact at 375, 768, 1024, and 1440 pixel widths with
no console errors, no external requests, visible keyboard focus, and no
horizontal overflow.

## Boundaries

- Always: preserve cell text by default, validate all limits before rendering,
  run focused and full checks before commits, and update English/Chinese public
  documentation together.
- Ask first: add a parser, tokenizer, network client, persistence, an export
  format, a release dependency, or an MCP transport.
- Never: silently repair data, evaluate formulas, emit SQL/YAML/Markdown in
  v0.1, use user input with `innerHTML`, transmit data, or claim an
  unimplemented Agent integration.

## Success Criteria

1. Valid RFC 4180 CSV and tab-delimited data parse identically in CLI and
   browser WASM paths.
2. Malformed quoting, inconsistent record width, explicit limits, and ambiguous
   dialects fail or diagnose loudly; no record is padded, truncated, or trimmed.
3. Formula-like values appear in inspection diagnostics and are escaped only by
   the named CSV/TSV export policy.
4. JSON export is stable and refuses empty or duplicate headers.
5. The browser makes no third-party request and renders adversarial headers and
   values as text, not markup or event handlers.
6. The repository meets Tinkora's bilingual README, CI, Pages, release, SBOM,
   checksum, provenance, and browser gates before it is published.
