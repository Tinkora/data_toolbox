# Tinkora Data Toolbox

面向开发者和 AI agent 操作者的本地 CSV/TSV 检查工具：在文件进入电子表格、脚本或工具链之前，先看清它的结构和风险。

[English](README.md)

<!-- markdownlint-disable MD033 -->
<p align="center">
  <a href="https://ko-fi.com/tinkora" target="_blank" rel="noopener noreferrer">
    <img
      src="https://ko-fi.com/img/githubbutton_sm.svg"
      alt="在 Ko-fi 上支持 Tinkora"
      width="520"
    >
  </a>
</p>
<!-- markdownlint-enable MD033 -->

## 为什么需要它

Miller、qsv、csvkit 和 VisiData 已经覆盖了成熟的通用表格处理能力。本项目保持窄范围：只报告结构和安全风险，不静默修改输入。当数据敏感、无法安装 CLI，或者打开电子表格反而不安全时，可以先用它做一次快速检查。

## 它能做什么

- 解析逗号、Tab、分号或竖线分隔的 CSV/TSV 类文本。
- 仅在结果明确时自动检测分隔符；是否存在表头必须显式选择。
- 报告行列数量、重复或空表头、记录列数不一致和疑似电子表格公式的单元格。
- 默认保留空格、带引号的换行、空记录和单元格原文。
- 输出确定性的 CSV、TSV 或 JSON；CSV/TSV 可以显式为疑似公式单元格添加单引号，降低导入电子表格时的风险。
- 通过原生 CLI 或本地浏览器 WASM 页面调用同一个 Rust core。

如果单元格在任意前导 ASCII 空格后的第一个字符是 `=`、`+`、`-`、`@`、Tab 或回车符，则会被判定为疑似公式；这会有意包含 `-42` 等值。`preserve` 策略不会修改任何单元格；显式选择 `escape_for_spreadsheet` 后，仅为匹配的 CSV/TSV 单元格添加一个前导单引号；JSON 始终保留原值。

## 它不会做什么

不会上传数据、联网、持久化、计算电子表格公式、自动修复，也不会转换 YAML/SQL/Markdown；没有 JavaScript parser fallback、MCP server 或 Agent-callable transport。结构化 JSON 只是机器可读格式，不代表已实现 Agent 集成。

v0.1 限制：输入 10 MiB、数据行 200,000 行、列 1,024 列、预览 100 行。输入必须是合法 UTF-8 文本（接受 UTF-8 BOM）。

## 快速开始

~~~bash
cargo run -p data_toolbox_cli -- inspect --delimiter auto --headers present data.csv
cargo run -p data_toolbox_cli -- convert --to json --headers present data.csv
~~~

文件参数省略或写为 `-` 时从 stdin 读取。错误以一个 JSON 对象写入 stderr；stdout 只包含请求的结果。成功时退出码为 `0`，CLI 语法或参数无效时为 `2`，输入、UTF-8、解析、转换或 I/O 失败时为 `1`。

## 浏览器工具

~~~bash
rustup toolchain install 1.85.0 --profile minimal --no-self-update
rustup target add wasm32-unknown-unknown --toolchain 1.85.0
rustup toolchain install 1.95.0 --profile minimal --no-self-update
cargo +1.95.0 install wasm-pack --version 0.15.0 --locked
cd crates/data_toolbox_web
npm ci --ignore-scripts
npx --no-install playwright install chromium
RUSTUP_TOOLCHAIN=1.85.0 npm run build:wasm
npm run test:wasm-smoke
~~~

除上述 Rust toolchain 外，浏览器流程还需要 Node.js 24 和 Python 3。Linux 用户需要按提示安装 Playwright 的系统依赖，或使用 `npx --no-install playwright install --with-deps chromium`。Smoke 测试使用真实 Chromium 验证 375、768、1024 和 1440 px。手动查看时，用本地 HTTP server 服务仓库根目录，再打开 `/crates/data_toolbox_web/web/`。

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
