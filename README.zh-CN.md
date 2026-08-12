# Tinkora Data Toolbox

面向开发者和 AI agent 操作者的本地 CSV/TSV 检查工具：在文件进入电子表格、脚本或工具链之前，先看清它的结构和风险。

[English](README.md)

## 为什么需要它

Miller、qsv、csvkit 和 VisiData 已经覆盖了成熟的通用表格处理能力。本项目保持窄范围：只报告结构和安全风险，不静默修改输入。当数据敏感、无法安装 CLI，或者打开电子表格反而不安全时，可以先用它做一次快速检查。

## 它能做什么

- 解析逗号、Tab、分号或竖线分隔的 CSV/TSV 类文本。
- 仅在结果明确时自动检测分隔符；是否存在表头必须显式选择。
- 报告行列数量、重复或空表头、记录列数不一致和疑似电子表格公式的单元格。
- 默认保留空格、带引号的换行、空记录和单元格原文。
- 输出确定性的 CSV、TSV 或 JSON；CSV/TSV 可以显式为疑似公式单元格添加单引号，降低导入电子表格时的风险。
- 通过原生 CLI 或本地浏览器 WASM 页面调用同一个 Rust core。

## 它不会做什么

不会上传数据、联网、持久化、计算电子表格公式、自动修复，也不会转换 YAML/SQL/Markdown；没有 JavaScript parser fallback、MCP server 或 Agent-callable transport。结构化 JSON 只是机器可读格式，不代表已实现 Agent 集成。

v0.1 限制：输入 10 MiB、数据行 200,000 行、列 1,024 列、预览 100 行。输入必须是合法 UTF-8 文本（接受 UTF-8 BOM）。

## 快速开始

~~~bash
cargo run -p data_toolbox_cli -- inspect --delimiter auto --headers present data.csv
cargo run -p data_toolbox_cli -- convert --to json --headers present data.csv
~~~

文件参数省略或写为 `-` 时从 stdin 读取。错误以一个 JSON 对象写入 stderr；stdout 只包含请求的结果。

## 浏览器工具

~~~bash
cd crates/data_toolbox_web
npm ci
npm run build:wasm
npm run test:wasm-smoke
~~~

Smoke 测试使用真实 Chromium 验证 375、768、1024 和 1440 px。手动查看时，用本地 HTTP server 服务仓库根目录，再打开 `/crates/data_toolbox_web/web/`。

## 开发检查

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

提交变更前请阅读 [CONTRIBUTING.zh-CN.md](CONTRIBUTING.zh-CN.md)、[SECURITY.zh-CN.md](SECURITY.zh-CN.md)、[SUPPORT.zh-CN.md](SUPPORT.zh-CN.md) 和[发布前检查清单](docs/RELEASE_CHECKLIST.zh-CN.md)。

## 成熟度与许可证

在 Tinkora 托管 CI 为本仓库生成证据之前，项目处于 **Draft**。能力标签和证据规则见 [docs/MATURITY.zh-CN.md](docs/MATURITY.zh-CN.md)。代码使用 [MIT License](LICENSE) 发布。

## 赞助

如果这个工具帮你节省了时间，可以在 Tinkora 组织发布 Buy Me a Coffee 页面后自愿支持维护者。赞助不是使用、支持或贡献的条件。
