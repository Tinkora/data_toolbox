# Release checklist

This checklist validates a candidate. It is not permission to publish.

- [ ] A second trusted reviewer checked the candidate, changelog, recovery plan, and compatibility impact.
- [ ] The exact clean commit, SemVer version, and immutable `v<version>` tag are recorded.
- [ ] `cargo fmt --all -- --check`, workspace tests, Clippy, locked WASM check, docs checks, and supply-chain checks pass.
- [ ] `npm ci --ignore-scripts`, `npm audit --audit-level=high`, WASM build, and all four Chromium viewports pass.
- [ ] English and Chinese public docs agree on limits, privacy, unsupported formats, and current maturity.
- [ ] Release artifacts have reproducible names, SHA-256 checksums, SPDX SBOM, license evidence, and provenance.
- [ ] GitHub hosted checks pass on the exact candidate commit; local success is not a substitute.
- [ ] Rollback or fix-forward owner, user notification, and prior known-good artifact are recorded.
- [ ] No tag, GitHub Release, package, or credential is created until the organization ruleset and protected environment permit it.
