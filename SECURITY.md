# Security Policy

## Supported Versions

Security fixes are applied to the latest released version. Please upgrade to the
most recent [release](https://github.com/kerryhatcher/rustywx/releases) before
reporting.

| Version | Supported |
|---------|-----------|
| latest  | ✅        |
| older   | ❌        |

## Reporting a Vulnerability

**Please do not open a public issue for security vulnerabilities.**

Report privately through either channel:

- GitHub's [private vulnerability reporting](https://github.com/kerryhatcher/rustywx/security/advisories/new)
  (**Security → Report a vulnerability**), or
- Email **kerry@kerryhatcher.com** with details and, if possible, steps to
  reproduce.

## What to Expect

- **Acknowledgement** within 7 days.
- An assessment of the report and, if confirmed, a fix timeline.
- Credit in the release notes once a fix ships, unless you prefer to remain
  anonymous.

## Scope

rustywx fetches data over the network from public NOAA/NWS/NHC endpoints and the
public `unidata-nexrad-level2` S3 bucket. It requires no credentials and stores
no secrets. Reports involving parsing untrusted network responses (malformed
Level II volumes, alert/tropical feeds) are in scope and especially welcome.
