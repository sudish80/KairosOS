# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

**Do not open public issues for security vulnerabilities.**

Send details to **security@kairosos.org**. You should receive a response within 48 hours.

### What to include:
- Description of the vulnerability
- Steps to reproduce
- Affected components and versions
- Potential impact
- Any suggested fix (if known)

### Process:
1. We acknowledge receipt within 48 hours.
2. We investigate and develop a fix.
3. We release a patched version and disclose responsibly.

## Scope

The following areas are in scope:
- All Rust daemons under `src/`
- All Python AI services under `ai/`
- All kernel modules under `kernel/`
- All shell scripts under `scripts/`
- Build and deployment infrastructure under `packaging/`, `config/`

## Disclosure Timeline

We aim to release fixes within 30 days of confirmation.
Coordinated disclosure with reporters is standard practice.
