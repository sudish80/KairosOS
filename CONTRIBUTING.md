# Contributing to KairosOS

Thank you for your interest in KairosOS! We welcome contributions from everyone.

## Code of Conduct

Be respectful, inclusive, and constructive. Harassment, trolling, and personal attacks will not be tolerated.

## Getting Started

1. **Fork the repository** and clone your fork.
2. **Set up your environment** — see [README.md](README.md#quick-start).
3. **Run the build** — `make rust-build && make kernel-build`.
4. **Run the tests** — `make test`.

## Development Workflow

### Branch Naming
- `feature/<name>` — New features
- `fix/<name>` — Bug fixes
- `refactor/<name>` — Refactoring
- `docs/<name>` — Documentation

### Commit Messages
```
<type>(<scope>): <brief description>

<optional body, 72 chars per line>
```
Types: `feat`, `fix`, `refactor`, `docs`, `test`, `ci`, `perf`, `chore`

### Pull Request Checklist
- [ ] Code compiles without warnings (`cargo clippy --deny warnings`)
- [ ] All tests pass (`cargo test --workspace && python3 -m pytest`)
- [ ] Rust code is formatted (`cargo fmt --check`)
- [ ] Python code is linted (`ruff check ai/`)
- [ ] Shell scripts pass shellcheck (`shellcheck scripts/**/*.sh`)
- [ ] New code includes tests
- [ ] Documentation updated (if applicable)

## Code Style

### Rust
- `#![deny(unsafe_code)]` in all crates
- Follow existing patterns (config/error/telemetry/worker module structure)
- Hardcoded paths for production, env vars for development overrides
- Use `tracing` for logging, `anyhow` for error handling

### Python
- Type hints required for all public functions
- Google-style docstrings
- JSON-RPC 2.0 over Unix sockets for inter-service communication
- Config via `Config` class with env var overrides (`KAIROS_<NAME>_*`)

### C (kernel modules)
- Follow Linux kernel coding style
- GPL-2.0-only license
- Use standard kernel APIs (`sysfs`, `miscdevice`, `ptp_clock_kernel`, etc.)

### Shell
- `set -euo pipefail` at the top of every script
- Timestamped logging via `log()` function
- `getopts` for CLI parsing
- Three-action dispatch: `status`, `enable`, `disable`

## Testing

- Rust: `cargo test --workspace` — unit tests per module
- Python: `pytest tests/ ai/*/tests/ -v` — handler tests
- Integration: `scripts/root/verify-build.sh` — full pipeline

## Security

- Never commit secrets, keys, or passwords
- Report vulnerabilities to `security@kairosos.org`
- All Rust code must be `#![deny(unsafe_code)]`
- Kernel modules must not introduce memory safety issues

## Questions?

Open an issue or reach out to the maintainers.
