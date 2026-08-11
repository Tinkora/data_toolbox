# ADR-0001: Start Data Toolbox With a Bounded CSV Inspector

## Status

Accepted

## Date

2026-08-12

## Context

The local `json_yaml_swiss`, `csv_sculptor`, and `md_porter` prototypes all
overstated their current capability. Their main functionality overlaps mature
tools such as `yq`, Dasel, csvkit, Miller, Pandoc, and VS Code Markdown PDF.
The prototypes also contain release-blocking correctness or security defects.

CSV handling still has a bounded and independently testable gap. Recent public
reports show that exporters can preserve attacker-controlled values beginning
with `=`, `+`, `-`, `@`, tab, or carriage return, which become live spreadsheet
formulas when opened. See [hyperfine #915](https://github.com/sharkdp/hyperfine/issues/915)
and [Axon #95](https://github.com/daxis-io/axon/issues/95). Users also need a
low-friction way to inspect one-off sensitive CSV/TSV files without uploading
them or installing a command-line data platform.

## Decision

Create one new `data_toolbox` repository. Its first release is a strict
CSV/TSV Inspector with a shared Rust core, a machine-readable CLI, and a local
WASM browser interface.

The initial feature set is deliberately limited to:

- quote-aware CSV/TSV parsing with explicit size, row, and column limits;
- delimiter confidence and structural diagnostics;
- first-row header mode selected explicitly by the caller;
- formula-like cell warnings;
- deterministic CSV, TSV, and JSON export;
- opt-in spreadsheet-formula escaping for CSV/TSV exports.

The first release does not transform data, infer types, generate SQL/YAML or
Markdown, repair malformed records, persist data, use a JavaScript fallback,
or claim MCP/Agent-callable support.

## Alternatives Considered

### Publish `csv_sculptor` unchanged

Rejected. It uses a different JavaScript mock when WASM is missing, exposes an
HTML/inline-handler injection path, silently changes row shape, and makes
unsafe SQL and formula-bearing spreadsheet exports.

### Publish a JSON/YAML/TOML converter first

Rejected for the first module. Existing local code loses YAML/TOML semantics,
and established offline tools already provide broader conversion. A later
Config Inspector can join Data Toolbox only with a strict loss policy.

### Publish a Markdown exporter first

Rejected for the first module. Safe HTML/PDF conversion has a wider attack
surface and a mature ecosystem. A later Markdown Inspector must reject or
escape executable content by default.

## Consequences

- The product launches with a narrow, testable contract rather than a generic
  data suite.
- Formula protection is explicit: inspection never changes data, while exports
  require a named preservation or escape policy.
- Duplicate or empty headers block JSON-object export rather than losing data.
- Subsequent modules share the repository only after their own go/no-go review.
