# KairosOS

[![License: GPL v2](https://img.shields.io/badge/License-GPL%20v2-blue.svg)](LICENSE)
[![CI](https://github.com/sudish80/KairosOS/actions/workflows/ci.yml/badge.svg)](https://github.com/sudish80/KairosOS/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/Python-3.12%2B-blue)](https://python.org)
[![Kernel](https://img.shields.io/badge/Kernel-6.x-critical)](https://kernel.org)

**KairosOS** is a production-hardened autonomous operating system with 20 Rust daemons, 8 Python AI microservices, 7 C kernel modules, 74 shell scripts, 35 systemd units, and 22 AppArmor profiles — **580+ source files, 24,000+ lines of code.**

```
┌─────────────────────────────────────────────────────────┐
│                     UI Layer                              │
│    kairos-tui (DRM/KMS)    kairos-web (dashboard)       │
├─────────────────────────────────────────────────────────┤
│                   AI / Reasoning                          │
│  confidence  context-manager  supervisor  hermes-agent   │
│  knowledge-graph  skill-evolver  task-scheduler          │
├─────────────────────────────────────────────────────────┤
│                Core Orchestration                         │
│  orchestrator  mesh  db  mcp  apply  git-logger          │
├─────────────────────────────────────────────────────────┤
│              Hardware / System Control                    │
│  bpf  fb  llm  recovery  avionics  robotics  vision      │
│  climate  finance  quantum  bio  build                    │
├─────────────────────────────────────────────────────────┤
│              Kernel Modules (7)                           │
│  dmverity  tpm  edac  ptp  fb  iommu  prochot            │
├─────────────────────────────────────────────────────────┤
│              System Management (74 scripts)               │
│  security/  network/  hardware/  storage/  self-heal/    │
└─────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Requirements
- **Rust 1.85+** — `rustup install stable`
- **Python 3.12+** — `python3 --version`
- **Linux 6.x kernel headers** — `sudo apt install linux-headers-$(uname -r)`
- **Docker** (optional, for ISO builds)

### Build & Install
```bash
git clone https://github.com/sudish80/KairosOS.git
cd KairosOS

# Build Rust daemons
make rust-build

# Build kernel modules
make kernel-build

# Run tests
make test

# Install
sudo make install
```

### Run
```bash
# Core services
sudo systemctl enable --now kairos-bpf kairos-mcp kairos-apply
sudo systemctl enable --now kairos-recovery kairos-git-logger

# OTA updates
sudo systemctl enable --now kairos-update-check.timer

# AI services (optional)
sudo systemctl enable --now kairos-confidence kairos-supervisor
```

## 📦 Architecture

| Layer | Components | Language | Files |
|-------|-----------|----------|-------|
| **Daemons** (20) | BPF, MCP, apply, git-logger, inference-hub, recovery, TUI, orchestrator, mesh, DB, framebuffer, LLM, build, climate, finance, vision, avionics, quantum, robotics, bio | Rust | 277 |
| **AI Services** (8) | confidence, context-manager, supervisor, hermes-agent, knowledge-graph, skill-evolver, task-scheduler, telemetry-collector | Python | 74 |
| **Kernel Modules** (7) | dm-verity, TPM, EDAC, PTP, framebuffer, IOMMU, PROCHOT | C | 13 |
| **Scripts** (74) | 22 security, 10 network, 26 hardware, 3 storage, 5 self-heal, 7 root, 1 tool | Bash | 77 |
| **Config** | Systemd (43 units), AppArmor (22 profiles), TOML (40 files) | Various | 100+ |

## 🔄 OTA Updates

KairosOS has a full over-the-air update system:
- **A/B partitions** with automatic rollback on 3 failed boots
- **Signed manifests** with GPG verification
- **HTTPS download** with SHA256 integrity checks
- **Delta updates** via bspatch (minimal bandwidth)
- **Staged rollouts** — deterministic hash-based canary deployment
- **Scheduled checks** via systemd timer (daily with randomized delay)

```bash
# Check for updates
sudo ota-update.sh check

# Apply update
sudo ota-update.sh apply && sudo reboot

# Rollback if needed
sudo ota-update.sh rollback && sudo reboot
```

## 🔒 Security

- **22 AppArmor profiles** — daemon-specific with least-privilege capabilities
- **dm-verity** — block-level integrity verification
- **TPM 2.0** — PCR binding and key sealing
- **Secure Boot** — signed initramfs and kernel modules
- **No arbitrary code execution** — `#![deny(unsafe_code)]` in all Rust crates

## 🧪 Testing

```bash
# Full build verification
sudo scripts/root/verify-build.sh

# Rust tests
cd src && cargo test --workspace

# Python tests
python3 -m pytest tests/ ai/*/tests/ -v

# Kernel modules
for d in kernel/*/; do make -C "$d"; done
```

## 📦 Packaging

```bash
make deb    # Debian/Ubuntu .deb
make rpm    # Fedora/RHEL .rpm
make arch   # Arch Linux package
```

## 🏗 Building ISO

```bash
make build-iso CONFIG=kairosos_defconfig
# Boot in QEMU:
make qemu-iso
```

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [Production Deployment](docs/production-deployment.md) | Full deployment guide |
| [OTA API Spec](docs/ota-server-api.md) | Update server API v1 |
| [CHECKLIST](CHECKLIST.md) | Complete feature status (1500 items) |
| [Architecture](docs/architecture-v2.md) | System architecture |

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

GNU General Public License v2.0 — see [LICENSE](LICENSE) for details.
