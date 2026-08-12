# Maturity and capability labels

The project is **Draft** until the hosted Tinkora workflows produce evidence for this exact repository and commit. Do not add a maturity badge before that evidence exists.

## Evidence levels

| Level | Minimum evidence |
| --- | --- |
| Draft | Scope, trust boundary, and representative local behavior are documented. |
| Alpha | Core success, invalid-input, boundary, failure, native, WASM, browser, documentation, and dependency checks pass in hosted CI. |
| Beta | Alpha evidence remains current; stable interfaces, non-maintainer external use, a closed feedback/remediation cycle, and a rehearsed recovery plan exist. |
| Stable | Beta evidence spans compatible releases; support lifecycle, two trusted release reviewers, and protected publishing controls are evidenced. |

## Capability labels

- **Human-usable:** a person can use the documented CLI or local browser UI.
- **Agent schema draft:** versioned machine-readable results exist, but no runnable agent transport or registration is promised.
- **Agent-callable:** a real transport and registration execute the contract; this project does not currently have one.
- **Dual-use:** both Human-usable and Agent-callable evidence exist; this project must not use this label in v0.1.

Maturity and capability are independent. A schema does not imply an Agent integration, and a local tool does not imply a hosted security boundary.
