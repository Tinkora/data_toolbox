# 发布前检查清单

本清单用于验证候选版本，不代表获准发布。

- [ ] 第二位可信审查者检查候选、CHANGELOG、恢复方案和兼容性影响。
- [ ] 已记录准确且干净的 commit、SemVer 版本和不可变 `v<version>` tag。
- [ ] `cargo fmt --all -- --check`、workspace 测试、Clippy、锁定的 WASM 检查、文档检查和供应链检查均通过。
- [ ] `npm ci --ignore-scripts`、`npm audit --audit-level=high`、WASM 构建以及四个 Chromium 视口均通过。
- [ ] 中英文公开文档对限制、隐私、不支持格式和当前成熟度的描述一致。
- [ ] 发布产物名称可复现，并具有 SHA-256 checksum、SPDX SBOM、许可证证据和 provenance。
- [ ] 托管 CI 在准确候选 commit 上通过；本地成功不能替代它。
- [ ] 已记录 rollback 或 fix-forward 的负责人、用户通知方式和上一份可用产物。
- [ ] 在组织 ruleset 和 protected environment 允许之前，不创建 tag、GitHub Release、package 或发布凭据。
