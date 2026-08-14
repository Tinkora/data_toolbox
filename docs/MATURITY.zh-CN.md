# 成熟度与能力标签

项目在 `v0.1.0-alpha.1` 时为 **Alpha**。tag commit 已有 native、WASM、浏览器、文档、安全和供应链托管证据。不得仅凭此版本推断 Beta 或 Stable 支持。

## 证据等级

| 等级 | 最低证据 |
| --- | --- |
| Draft | 已记录范围、信任边界和可运行的本地代表性行为。 |
| Alpha | core 成功、无效输入、边界、失败、native、WASM、浏览器、文档和依赖检查在托管 CI 中通过。 |
| Beta | Alpha 证据持续有效；接口稳定，有非维护者外部使用记录，完成一个反馈/修复闭环，并演练恢复方案。 |
| Stable | Beta 证据跨兼容版本持续；支持周期、两位可信发布审查者和受保护发布控制均有证据。 |

## 能力标签

- **Human-usable：** 人可以使用文档所述的 CLI 或本地浏览器 UI。
- **Agent schema draft：** 已有带版本的机器可读结果，但没有承诺可运行的 Agent transport 或注册。
- **Agent-callable：** 存在真实 transport 和 registration 并执行契约；本项目 v0.1 没有这一能力。
- **Dual-use：** 同时有 Human-usable 和 Agent-callable 证据；v0.1 不得使用此标签。

成熟度与调用能力相互独立。schema 不代表 Agent 集成，本地工具也不代表托管安全边界。
