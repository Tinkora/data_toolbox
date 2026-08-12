# Tinkora Data Toolbox

Local CSV/TSV inspection for people who need to understand a file before it reaches a spreadsheet, script, or AI-agent workflow.

[简体中文](README.zh-CN.md)

## Why this exists

Mature tools such as Miller, qsv, csvkit, and VisiData already cover broad table processing. This project stays intentionally narrow: it reports shape and safety risks without silently changing the input. It is useful when data is sensitive, a CLI is unavailable, or a quick browser check is safer than opening a file in a spreadsheet.

## What it does

- Parse CSV or TSV-like text with comma, tab, semicolon, or pipe delimiters.
- Detect a delimiter only when the result is unambiguous; header presence is always explicit.
- Report row/column counts, duplicate or empty headers, jagged records, and spreadsheet-formula-like cells.
- Preserve spaces, quoted newlines, empty records, and cell text by default.
- Export deterministic CSV, TSV, or JSON. CSV/TSV can explicitly prefix formula-like cells with an apostrophe for spreadsheet import.
- Run the same Rust core through a native CLI or a local browser WASM page.

## What it does not do

There is no upload, network request, persistence, spreadsheet evaluation, automatic repair, YAML/SQL/Markdown conversion, JavaScript parser fallback, MCP server, or Agent-callable transport. The structured JSON shape is machine-readable; it is not a claim of agent integration.

The v0.1 limits are 10 MiB input, 200,000 data rows, 1,024 columns, and a 100-row preview. Input must be valid UTF-8 text (a UTF-8 BOM is accepted).

## Quick start

~~~bash
cargo run -p data_toolbox_cli -- inspect --delimiter auto --headers present data.csv
cargo run -p data_toolbox_cli -- convert --to json --headers present data.csv
~~~

Use `-` or omit the file to read stdin. Errors are one JSON object on stderr; stdout contains only the requested result.

## Browser tool

~~~bash
cd crates/data_toolbox_web
npm ci
npm run build:wasm
npm run test:wasm-smoke
~~~

The smoke suite uses real Chromium at 375, 768, 1024, and 1440 pixels. To open the page manually, serve the repository root with a local HTTP server and open `/crates/data_toolbox_web/web/`.

## Development checks

~~~bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p data_toolbox_web --target wasm32-unknown-unknown --locked
npx --yes markdownlint-cli2@0.23.2 "**/*.md"
ruby scripts/test_check_docs.rb
ruby scripts/check_docs.rb
ruby scripts/test_check_workflow_contracts.rb
ruby scripts/check_workflow_contracts.rb
~~~

See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), [SUPPORT.md](SUPPORT.md), and the [release checklist](docs/RELEASE_CHECKLIST.md) before proposing a change or release.

## Maturity and license

The project is currently **Draft** until the hosted Tinkora CI baseline has produced evidence for this repository. Capability labels and evidence rules are documented in [docs/MATURITY.md](docs/MATURITY.md). The code is released under the [MIT License](LICENSE).

## Sponsorship

If this tool saves you time, support the maintainers through the Tinkora organization's Buy Me a Coffee page when one is published. Sponsorship is optional and never a condition of use, support, or contribution.
