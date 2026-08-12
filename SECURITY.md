# Security Policy

## Supported versions

The project is not released yet. Until the first hosted release, security fixes are applied to `main`.

| Version | Supported |
| --- | --- |
| `main` | Yes |
| Unmodified local copies | No commitment |

## Reporting a vulnerability

Do not open a public issue for a vulnerability, malicious CSV, secret, or private input. After the repository is hosted, use GitHub's private vulnerability reporting channel if it is enabled. If it is not enabled, contact the Tinkora maintainers through a private GitHub channel and include the affected commit, impact, reproduction steps, and suggested mitigation.

Do not include real credentials or private user data. We will acknowledge a credible report, coordinate a fix, and publish only the minimum information needed for users to assess impact.

## Security boundary

The v0.1 tool is local-only: input is processed in memory, no network API or persistence exists, and the browser renders untrusted values with DOM text APIs. New transports, uploads, persistence, formula evaluation, or package publishing require a separate threat-model and review.
