# 贡献指南

感谢你改进 Tinkora 的实用小工具。如果提案会改变产品边界、解析行为、隐私边界、依赖集合或发布产物，请先创建 issue 或 discussion。

## 开始编码前

1. 阅读 [README](README.zh-CN.md)、[安全策略](SECURITY.zh-CN.md) 和[成熟度规则](docs/MATURITY.zh-CN.md)。
2. 确认变更解决的是可复现的用户问题，而不是重复成熟的通用 CSV 工具。
3. 创建聚焦的分支或 worktree。
4. 改变行为时，先写面向结果且会失败的测试。

## 实现规则

- 保持解析、诊断、导出、CLI I/O 和浏览器渲染的现有边界。
- 不得静默 trim、补齐、截断、去重或修复用户数据。
- 浏览器代码必须调用 Rust/WASM core；没有 JavaScript parser fallback、网络请求、持久化，也不得用 `innerHTML` 渲染输入。
- 公开文档、代码注释和提交信息使用英文；公开行为变化时同步更新完整中文 README/文档。
- 前端 HTML 变更必须使用 `ui-ux-pro-max` 生成设计系统并做可访问性审查，同时在 375、768、1024 和 1440 px 做真实浏览器检查。

## 检查与 Pull Request

运行 [README 开发检查](README.zh-CN.md#开发检查) 中的命令。不要提交生成的 `target/`、`pkg/`、`node_modules/` 和 Playwright 产物。Pull Request 应说明用户问题、范围、测试证据、隐私影响和文档变更。每个提交保持一个逻辑变更，并使用英文 Conventional Commits。

维护者会审查正确性、安全性、可访问性、依赖影响，以及成熟度/能力声明的证据。CI 通过是必要条件，但不能单独证明新的使用场景有价值。
