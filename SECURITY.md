# Security Policy

## Supported versions

The project is not released yet. Until the first hosted release, security fixes are applied to `main`.

| Version | Supported |
| --- | --- |
| `main` | Yes |
| Unmodified local copies | No commitment |

## Reporting a vulnerability

Do not open a public issue for a vulnerability, malicious CSV, secret, or private input. Use [GitHub Private Vulnerability Reporting](https://github.com/Tinkora/data_toolbox/security/advisories/new) and include the affected commit, impact, reproduction steps, and suggested mitigation. This private channel is enabled and is a publication requirement for the repository.

Do not include real credentials or private user data. We will acknowledge a credible report, coordinate a fix, and publish only the minimum information needed for users to assess impact.

## Security boundary

The v0.1 tool is local-only: input is processed in memory, no network API or persistence exists, and the browser renders untrusted values with DOM text APIs. New transports, uploads, persistence, formula evaluation, or package publishing require a separate threat-model and review.
