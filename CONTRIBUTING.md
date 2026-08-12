# Contributing

Thank you for improving a small, practical Tinkora tool. Start with an issue or discussion when a proposal changes the product boundary, parser behavior, privacy boundary, dependency set, or release artifacts.

## Before coding

1. Read the [README](README.md), [security policy](SECURITY.md), and [maturity rules](docs/MATURITY.md).
2. Confirm the change addresses a reproducible user problem and does not duplicate a mature general-purpose CSV tool.
3. Create a focused branch or worktree.
4. For behavior changes, write a failing outcome-focused test first.

## Implementation rules

- Keep parsing, diagnostics, exports, CLI I/O, and browser rendering in their existing boundaries.
- Do not silently trim, pad, truncate, deduplicate, or repair user data.
- Browser code must call the Rust/WASM core. There is no JavaScript parser fallback, network request, persistence, or `innerHTML` rendering of input.
- Keep public docs, code comments, and commit messages in English. Update the complete Chinese README/docs counterpart when public behavior changes.
- Frontend HTML changes require `ui-ux-pro-max` design-system and accessibility review, plus real browser checks at 375, 768, 1024, and 1440 px.

## Checks and pull requests

Run the commands in the [README development checks](README.md#development-checks). Keep generated `target/`, `pkg/`, `node_modules/`, and Playwright artifacts out of commits. A pull request should explain the user problem, scope, test evidence, privacy impact, and documentation changes. Keep one logical change per commit and use English Conventional Commits.

Maintainers review correctness, security, accessibility, dependency impact, and the evidence behind any maturity or capability claim. Passing CI is necessary but does not by itself prove a new use case is valuable.
