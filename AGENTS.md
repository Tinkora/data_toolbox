# Data Toolbox Guide

## Purpose

`data_toolbox` starts with a strict CSV/TSV Inspector. It is a local browser
and CLI utility for inspecting tabular text and producing explicit, safe
exports. It is not a spreadsheet, database client, general data-processing
platform, or MCP server.

## Public Conventions

- Use English for public documentation, code comments, and commit messages.
- Keep `README.md` as the default entry point and maintain a complete
  `README.zh-CN.md` counterpart.
- Follow Conventional Commits for public commits.
- Do not describe a schema, WASM export, or CLI as MCP or Agent-callable unless
  a runnable transport, installation instructions, permission boundaries, and
  end-to-end integration tests exist.

## Product Contract

- Preserve input field text exactly unless an explicitly requested export
  policy changes it.
- Never silently trim, pad, truncate, deduplicate, or repair rows.
- Treat invalid CSV, duplicate/empty headers, ambiguous delimiters, and
  spreadsheet-formula-like cells as explicit diagnostics or errors.
- Never make network requests, persist user data, or use a JavaScript parser
  fallback for the browser tool.

## Required Checks

Run the documented Rust, WebAssembly, documentation, and real-browser checks
before each milestone commit. Before creating, modifying, reviewing, or
debugging user-facing HTML, invoke `ui-ux-pro-max` and validate 375, 768,
1024, and 1440 pixel viewports in a real browser.
