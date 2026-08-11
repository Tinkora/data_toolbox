# CSV Inspector Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` (recommended) or
> `superpowers:executing-plans` to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a strict local CSV/TSV inspector with deterministic safe exports,
a machine-readable CLI, and a real Rust/WASM browser interface.

**Architecture:** `data_toolbox_core` owns every input rule, diagnostic, and
export. `data_toolbox_cli` and `data_toolbox_web` are thin adapters that cannot
silently change parsing behavior. The browser uses the real WASM artifact and
DOM text APIs; there is no JavaScript parser fallback.

**Tech Stack:** Rust 2024 (MSRV 1.85), `csv`, `serde`, `serde_json`,
`thiserror`, `wasm-bindgen`, Node.js 24, Playwright 1.54, plain HTML/CSS/JS.

---

## File Map

- `Cargo.toml`: workspace members, shared versions, release profile.
- `crates/data_toolbox_core/src/model.rs`: public options, table, result, and
  diagnostic types.
- `crates/data_toolbox_core/src/error.rs`: stable failures and error codes.
- `crates/data_toolbox_core/src/inspect.rs`: limits, dialect selection, strict
  record parsing, ordered diagnostics.
- `crates/data_toolbox_core/src/export.rs`: deterministic CSV/TSV/JSON output
  and explicit formula policy.
- `crates/data_toolbox_cli/src/main.rs`: argument parsing and stdin/file I/O.
- `crates/data_toolbox_web/src/lib.rs`: serialization-only WASM boundary.
- `crates/data_toolbox_web/web/`: browser interface using the WASM boundary.
- `crates/data_toolbox_web/tests/browser/inspector.spec.js`: real browser
  workflow, privacy, accessibility, and viewport checks.
- `scripts/`: offline documentation and workflow contract checks.

## Task 1: Workspace and Stable Core Contract

**Files:**

- Create: `Cargo.toml`
- Create: `crates/data_toolbox_core/Cargo.toml`
- Create: `crates/data_toolbox_core/src/lib.rs`
- Create: `crates/data_toolbox_core/src/model.rs`
- Create: `crates/data_toolbox_core/src/error.rs`
- Test: `crates/data_toolbox_core/tests/contract.rs`

- [ ] **Step 1: Write the failing serialized-contract test**

```rust
#[test]
fn inspection_uses_versioned_snake_case_contract() {
    let inspection = data_toolbox_core::Inspection {
        schema_version: data_toolbox_core::SCHEMA_VERSION,
        delimiter: String::new(),
        headers: Vec::new(),
        row_count: 0,
        column_count: 0,
        preview_rows: Vec::new(),
        diagnostics: Vec::new(),
    };
    let value =
        serde_json::to_value(inspection).expect("inspection serializes");
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["row_count"], 0);
    assert!(value.get("schemaVersion").is_none());
}
```

- [ ] **Step 2: Run the test and prove RED**

Run: `cargo test -p data_toolbox_core --test contract`

Expected: failure because the workspace and `Inspection` do not exist.

- [ ] **Step 3: Add the minimum public model**

```rust
pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_INPUT_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_ROWS: usize = 200_000;
pub const MAX_COLUMNS: usize = 1_024;
pub const MAX_PREVIEW_ROWS: usize = 100;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DelimiterMode { Auto, Comma, Tab, Semicolon, Pipe }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HeaderMode { Present, Absent }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity { Info, Warning }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub row: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Inspection {
    pub schema_version: u32,
    pub delimiter: String,
    pub headers: Vec<String>,
    pub row_count: usize,
    pub column_count: usize,
    pub preview_rows: Vec<Vec<String>>,
    pub diagnostics: Vec<Diagnostic>,
}
```

Define `CoreError` variants for all fatal codes in the design and expose
`code() -> &'static str`. Keep `data_toolbox_core` free of WASM dependencies.
The public model also defines `InspectOptions`, `ConvertOptions`,
`OutputFormat`, `FormulaPolicy`, and `Conversion`; their serialized fields use
`snake_case`.
`ConvertOptions` contains `delimiter`, `headers`, `output`, and
`formula_policy`. Add `INVALID_OPTIONS` as the stable error code for malformed
adapter options.

- [ ] **Step 4: Prove GREEN and run the first core gate**

Run:

```text
cargo test -p data_toolbox_core --test contract
cargo fmt --all -- --check
cargo clippy -p data_toolbox_core --all-targets -- -D warnings
```

Expected: all commands succeed without warnings.

- [ ] **Step 5: Commit the contract milestone**

```text
git add Cargo.toml Cargo.lock crates/data_toolbox_core
git commit -m "feat: define the CSV inspector contract"
```

## Task 2: Strict Inspection and Ordered Diagnostics

**Files:**

- Create: `crates/data_toolbox_core/src/inspect.rs`
- Modify: `crates/data_toolbox_core/src/lib.rs`
- Modify: `crates/data_toolbox_core/src/model.rs`
- Test: `crates/data_toolbox_core/tests/inspect.rs`

- [ ] **Step 1: Add failing outcome tests**

Add this explicit test helper before the tests shown below so these fixtures
never depend on automatic dialect selection:

```rust
fn present_headers() -> InspectOptions {
    InspectOptions {
        delimiter: DelimiterMode::Comma,
        headers: HeaderMode::Present,
    }
}
```

```rust
#[test]
fn quoted_newline_and_spaces_are_preserved() {
    let input = "name,note\r\nAlice,\"  first\nsecond  \"\r\n";
    let result = inspect(input, &present_headers()).expect("valid CSV");
    assert_eq!(result.preview_rows[0][1], "  first\nsecond  ");
}

#[test]
fn jagged_record_is_rejected_without_padding_or_truncation() {
    let error = inspect("a,b\n1\n", &present_headers()).unwrap_err();
    assert_eq!(error.code(), "ROW_WIDTH_MISMATCH");
}

#[test]
fn duplicate_headers_and_formula_cells_are_reported_in_order() {
    let result = inspect(
        "name,name\nAlice,=1+1\n",
        &present_headers(),
    )
    .unwrap();
    let codes: Vec<_> = result
        .diagnostics
        .iter()
        .map(|item| item.code.as_str())
        .collect();
    assert_eq!(codes, ["DUPLICATE_HEADER", "FORMULA_LIKE_CELL"]);
}
```

Add separate tests for BOM, CRLF, comma/tab/semicolon/pipe, ambiguous dialect,
invalid quotes, empty headers, 10 MiB, 200,000 rows, 1,024 columns, and preview
truncation without data truncation.

- [ ] **Step 2: Run focused tests and prove RED**

Run: `cargo test -p data_toolbox_core --test inspect`

Expected: compile or assertion failures because `inspect` is absent.

- [ ] **Step 3: Implement strict parsing**

```rust
pub fn inspect(
    input: &str,
    options: &InspectOptions,
) -> Result<Inspection, CoreError> {
    validate_input_size(input)?;
    let delimiter = select_delimiter(input, &options.delimiter)?;
    let table = parse_records(
        input.trim_start_matches('\u{feff}'),
        delimiter,
        &options.headers,
    )?;
    validate_shape(&table)?;
    Ok(build_inspection(table, delimiter))
}
```

Evaluate auto-detection candidates with the `csv` parser, not character counts.
Select a candidate only when it has a consistent width greater than one and a
strictly better score. Return `AMBIGUOUS_DELIMITER` on a tied best score. Use
`csv::Trim::None` and never mutate record width.

- [ ] **Step 4: Prove GREEN and run the core gate**

Run:

```text
cargo test -p data_toolbox_core --test inspect
cargo test -p data_toolbox_core
cargo clippy -p data_toolbox_core --all-targets -- -D warnings
```

Expected: all inspection fixtures pass with deterministic diagnostic order.

- [ ] **Step 5: Commit strict inspection**

```text
git add crates/data_toolbox_core
git commit -m "feat: inspect CSV without silent data changes"
```

## Task 3: Deterministic and Formula-Aware Export

**Files:**

- Create: `crates/data_toolbox_core/src/export.rs`
- Modify: `crates/data_toolbox_core/src/lib.rs`
- Modify: `crates/data_toolbox_core/src/model.rs`
- Test: `crates/data_toolbox_core/tests/export.rs`

- [ ] **Step 1: Add failing export tests**

Add explicit helpers before the tests. They keep conversion fixtures focused on
the policy under test rather than auto-detection:

```rust
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
```

```rust
#[test]
fn preserve_policy_reports_but_does_not_change_formula_text() {
    let output = convert(
        "value\n=1+1\n",
        &csv_options(FormulaPolicy::Preserve),
    )
    .unwrap();
    assert!(output.content.contains("=1+1"));
    assert!(output
        .diagnostics
        .iter()
        .any(|item| item.code == "FORMULA_LIKE_CELL"));
}

#[test]
fn explicit_spreadsheet_policy_prefixes_formula_like_cells() {
    let output = convert(
        "value\n  @SUM(A1)\n",
        &csv_options(FormulaPolicy::EscapeForSpreadsheet),
    ).unwrap();
    assert!(output.content.contains("'  @SUM(A1)"));
}

#[test]
fn json_refuses_duplicate_headers() {
    let error = convert("x,x\n1,2\n", &json_options()).unwrap_err();
    assert_eq!(error.code(), "JSON_EXPORT_REQUIRES_UNIQUE_HEADERS");
}
```

Also test commas, quotes, CRLF, newlines, empty last fields, TSV quoting,
deterministic JSON key order, absent headers, and empty headers.

- [ ] **Step 2: Run focused tests and prove RED**

Run: `cargo test -p data_toolbox_core --test export`

Expected: failure because `convert` and exporters do not exist.

- [ ] **Step 3: Implement minimal exporters**

```rust
fn protect_formula(value: &str, policy: FormulaPolicy) -> Cow<'_, str> {
    if policy == FormulaPolicy::EscapeForSpreadsheet && is_formula_like(value) {
        Cow::Owned(format!("'{value}"))
    } else {
        Cow::Borrowed(value)
    }
}

pub fn convert(
    input: &str,
    options: &ConvertOptions,
) -> Result<Conversion, CoreError> {
    let parsed = parse_for_conversion(input, options)?;
    let content = match options.output {
        OutputFormat::Csv => {
            write_delimited(&parsed, b',', options.formula_policy)?
        }
        OutputFormat::Tsv => {
            write_delimited(&parsed, b'\t', options.formula_policy)?
        }
        OutputFormat::Json => write_json_rows(&parsed)?,
    };
    Ok(Conversion::new(options.output.clone(), content, parsed.diagnostics))
}
```

Use `csv::Writer` for CSV and TSV. Build JSON objects with
`serde_json::Map` in header order. Never use `HashMap` for exported objects.

- [ ] **Step 4: Prove GREEN and commit**

Run:

```text
cargo test -p data_toolbox_core --test export
cargo test -p data_toolbox_core
cargo fmt --all -- --check
cargo clippy -p data_toolbox_core --all-targets -- -D warnings
```

Expected: all commands succeed.

Commit:

```text
git add crates/data_toolbox_core
git commit -m "feat: add deterministic safe CSV exports"
```

## Task 4: Native Machine-Readable CLI

**Files:**

- Modify: `Cargo.toml`
- Create: `crates/data_toolbox_cli/Cargo.toml`
- Create: `crates/data_toolbox_cli/src/main.rs`
- Test: `crates/data_toolbox_cli/tests/cli.rs`

- [ ] **Step 1: Add failing CLI process tests**

```rust
#[test]
fn inspect_reads_stdin_and_writes_versioned_json() {
    let mut command = Command::cargo_bin("data-toolbox").unwrap();
    command.args(["inspect", "--delimiter", "comma", "--headers", "present"])
        .write_stdin("name,role\nAlice,owner\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\":1"));
}

#[test]
fn malformed_input_uses_json_stderr_and_nonzero_exit() {
    let mut command = Command::cargo_bin("data-toolbox").unwrap();
    command.args(["inspect"]).write_stdin("a,b\n\"broken\n")
        .assert().failure()
        .stderr(predicate::str::contains("\"code\":\"INVALID_CSV\""));
}
```

- [ ] **Step 2: Run tests and prove RED**

Run: `cargo test -p data_toolbox_cli --test cli`

Expected: failure because the binary does not exist.

- [ ] **Step 3: Implement explicit argument and I/O boundaries**

```rust
fn main() -> ExitCode {
    match run(
        std::env::args_os().skip(1),
        std::io::stdin(),
        std::io::stdout(),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", error.to_json());
            ExitCode::FAILURE
        }
    }
}
```

Reject unknown flags and missing values. Read only the named file or stdin.
Do not use environment-based defaults, recursive paths, network APIs, or
shell execution. Use `assert_cmd` and `predicates` only as dev-dependencies.

- [ ] **Step 4: Prove GREEN and commit**

Run:

```text
cargo test -p data_toolbox_cli --test cli
cargo test --workspace --locked
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: stdin, file, success, and error-envelope tests pass.

Commit:

```text
git add Cargo.toml Cargo.lock crates/data_toolbox_cli
git commit -m "feat: expose CSV inspection through a native CLI"
```

## Task 5: Real WASM Browser Interface

**Prerequisite:** Invoke `ui-ux-pro-max`, run its `--design-system` search for
the inspector, then run relevant HTML and accessibility searches before editing.

**Files:**

- Modify: `Cargo.toml`
- Create: `crates/data_toolbox_web/Cargo.toml`
- Create: `crates/data_toolbox_web/src/lib.rs`
- Create: `crates/data_toolbox_web/package.json`
- Create: `crates/data_toolbox_web/package-lock.json`
- Create: `crates/data_toolbox_web/playwright.config.js`
- Create: `crates/data_toolbox_web/web/index.html`
- Create: `crates/data_toolbox_web/web/styles.css`
- Create: `crates/data_toolbox_web/web/main.js`
- Test: `crates/data_toolbox_web/tests/browser/inspector.spec.js`

- [ ] **Step 1: Add failing browser and WASM contract tests**

```javascript
test('renders adversarial CSV as text', async ({ page }) => {
  const requests = [];
  page.on('request', request => requests.push(request.url()));
  await page.goto('/web/');
  await page
    .getByLabel('CSV or TSV input')
    .fill('name\n<img src=x onerror=alert(1)>\n');
  await page.getByRole('button', { name: 'Inspect data' }).click();
  await expect(
    page.getByRole('cell', { name: '<img src=x onerror=alert(1)>' }),
  ).toBeVisible();
  expect(await page.evaluate(() => window.__executedMarkup)).toBeUndefined();
  const origin = new URL(page.url()).origin;
  expect(requests.filter(url => new URL(url).origin !== origin)).toEqual([]);
});
```

Add all four viewports, keyboard focus, upload, paste, malformed input,
diagnostics, preserve/escape exports, no horizontal page overflow, and zero
console errors. Fail the test if WASM initialization does not reach `Ready`.

- [ ] **Step 2: Build the empty boundary and prove RED**

Run:

```text
cargo check -p data_toolbox_web --target wasm32-unknown-unknown
cd crates/data_toolbox_web && npm ci && npm run test:wasm-smoke
```

Expected: the Rust boundary can be scaffolded, but the browser flow fails
because the UI behavior is absent.

- [ ] **Step 3: Implement the thin WASM boundary**

```rust
#[wasm_bindgen]
pub fn inspect_csv(input: &str, options_json: &str) -> JsValue {
    let response = decode::<InspectOptions>(options_json)
        .and_then(|options| data_toolbox_core::inspect(input, &options));
    respond(response)
}

#[wasm_bindgen]
pub fn convert_csv(input: &str, options_json: &str) -> JsValue {
    let response = decode::<ConvertOptions>(options_json)
        .and_then(|options| data_toolbox_core::convert(input, &options));
    respond(response)
}
```

Return `{ "ok": true, "data": ... }` or
`{ "ok": false, "error": { "code": ..., "message": ... } }`.
`decode` deserializes adapter options and maps invalid JSON or enum values to
`CoreError::invalid_options`; `respond` serializes either envelope with
`serde_wasm_bindgen` without panicking.

- [ ] **Step 4: Implement the work-focused interface**

Use semantic labels, a compact toolbar, input and preview panes, diagnostic
list, and export controls. Create table cells with `document.createElement`
and `textContent`; never use `innerHTML` with any result or input value. Load
only local CSS, JS, and WASM. Show a visible initialization/error state and do
not provide fallback parsing.

- [ ] **Step 5: Prove GREEN and commit**

Run:

```text
cargo check -p data_toolbox_web --target wasm32-unknown-unknown --locked
cd crates/data_toolbox_web
npm run build:wasm
npm run test:wasm-smoke
```

Expected: Chromium passes at 375, 768, 1024, and 1440 px with real WASM,
no external requests, no console errors, and no page overflow.

Commit:

```text
git add Cargo.toml Cargo.lock crates/data_toolbox_web
git commit -m "feat: add the local CSV inspector web app"
```

## Task 6: Public Documentation and Delivery Automation

**Files:**

- Create: `README.md`, `README.zh-CN.md`, `LICENSE`, `CHANGELOG.md`
- Create: `CONTRIBUTING.md`, `CONTRIBUTING.zh-CN.md`
- Create: `SECURITY.md`, `SECURITY.zh-CN.md`
- Create: `SUPPORT.md`, `SUPPORT.zh-CN.md`
- Create: `docs/MATURITY.md`, `docs/MATURITY.zh-CN.md`
- Create: `docs/RELEASE_CHECKLIST.md`, `docs/RELEASE_CHECKLIST.zh-CN.md`
- Create: `.github/dependabot.yml`
- Create: `.github/workflows/quality.yml`
- Create: `.github/workflows/docs-quality.yml`
- Create: `.github/workflows/supply-chain.yml`
- Create: `.github/workflows/pages.yml`
- Create: `.github/workflows/release.yml`
- Create: `scripts/check_docs.rb`, `scripts/check_workflow_contracts.rb`
- Test: `scripts/test_check_docs.rb`, `scripts/test_check_workflow_contracts.rb`

- [ ] **Step 1: Copy and fail the template contract tests**

Copy the organization template checks, rename every crate/path/product token,
and run:

```text
ruby scripts/test_check_docs.rb
ruby scripts/check_docs.rb
ruby scripts/test_check_workflow_contracts.rb
ruby scripts/check_workflow_contracts.rb
```

Expected before docs/workflows exist: at least one check fails on a missing
required public file or reusable workflow call.

- [ ] **Step 2: Add truthful bilingual documentation**

Document exact limits, the preserve/escape formula policy, unsupported
encodings and formats, CLI exit behavior, browser privacy boundary, maturity,
and the difference between machine-readable and Agent-callable. Keep English
as `README.md`; provide a complete `README.zh-CN.md` with the same commands and
claims. Do not publish `skills/mcp-tools.json`.

- [ ] **Step 3: Add pinned reusable workflows**

Use full commit SHAs for Tinkora reusable Rust, WASM, supply-chain, Pages, and
release workflows. Give each job the minimum permissions. Release only a
version-matched tag, checksums, SPDX SBOM, license report, provenance, and the
platform archives required by the CLI.

- [ ] **Step 4: Run the complete local release gate**

Run:

```text
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p data_toolbox_web --target wasm32-unknown-unknown --locked
cd crates/data_toolbox_web && npm ci && npm audit --audit-level=high
npm run build:wasm && npm run test:wasm-smoke
cd ../..
npx --yes markdownlint-cli2@0.23.2 "**/*.md"
ruby scripts/test_check_docs.rb
ruby scripts/check_docs.rb
ruby scripts/test_check_workflow_contracts.rb
ruby scripts/check_workflow_contracts.rb
```

Expected: every command succeeds; npm reports zero high or critical
vulnerabilities; four browser projects pass.

- [ ] **Step 5: Commit the public delivery baseline**

```text
git add .github README.md README.zh-CN.md LICENSE CHANGELOG.md \
  CONTRIBUTING* SECURITY* SUPPORT* docs scripts deny.toml
git commit -m "chore: add the public delivery baseline"
```

## Task 7: GitHub Publication and Release

**Files:**

- Modify after successful release: workspace `TOOL_MATRIX.md`
- Modify after successful release: workspace `TINKORA_ROADMAP.md`
- Modify after successful release: `Tinkora/.github` organization profile and
  audit policy files.

- [ ] **Step 1: Reverify the publishing identity and clean tree**

Run:

```text
gh auth status -h github.com
git status --short --branch
```

Expected: active account is `tinkeragora`; local `main` is clean.

- [ ] **Step 2: Create and configure the public repository**

Create `Tinkora/data_toolbox`, push `main`, set the description/homepage/topics,
enable the intended interaction and security features, and add the same
delete/force-push and immutable `v*` tag rulesets used by current Tinkora tools.

- [ ] **Step 3: Verify hosted CI and Pages before tagging**

Require successful Quality, CodeQL, Documentation quality, Supply chain, and
Pages runs for the exact target commit. Open Pages at all four viewports and
verify the served WASM MIME type and no external requests.

- [ ] **Step 4: Publish and independently verify `v0.1.0`**

Create the annotated delivery tag only after hosted gates pass. Verify
downloaded archives, SHA-256 values, SPDX JSON, license JSON, and GitHub
attestations.

- [ ] **Step 5: Update portfolio facts and commit each repository**

Record only the verified version, URLs, run IDs, assets, and remaining feedback
gate. Do not publish the rejected prototype history or internal migration
process. Use English Conventional Commits in public repositories.
