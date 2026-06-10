# KairosOS Architecture

## Overview

KairosOS is an AI-native operating system built on three foundational projects:

```
┌─────────────────────────────────────────────────────┐
│                    KairosOS                          │
│  ┌──────────────────────────────────────────────┐   │
│  │           User Experience Layer               │   │
│  │  ┌─────────┐ ┌──────────┐ ┌──────────────┐  │   │
│  │  │Console  │ │Web       │ │Telegram/     │  │   │
│  │  │TUI      │ │Dashboard │ │Discord/etc.  │  │   │
│  │  └─────────┘ └──────────┘ └──────────────┘  │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │            Hermes Agent (AI Core)             │   │
│  │  ┌─────────┐ ┌──────────┐ ┌──────────────┐  │   │
│  │  │Learning │ │Memory    │ │Skill         │  │   │
│  │  │Loop     │ │(FTS5)    │ │Creation      │  │   │
│  │  └─────────┘ └──────────┘ └──────────────┘  │   │
│  │  ┌─────────┐ ┌──────────┐ ┌──────────────┐  │   │
│  │  │Subagent │ │Gateway   │ │Cron          │  │   │
│  │  │Delegate │ │Multi-Chn │ │Scheduler     │  │   │
│  │  └─────────┘ └──────────┘ └──────────────┘  │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │         OpenClaw Gateway (Optional)           │   │
│  │  Multi-channel bridge + multi-agent routing  │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │         KairosOS System Tools & Skills        │   │
│  │  System Manager • Package Manager • Network  │   │
│  │  Service Manager • Security Audit • Docker   │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │       Linux Kernel (torvalds/linux)           │   │
│  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌────────────┐ │   │
│  │  │eBPF  │ │CGroup│ │NS    │ │Security    │ │   │
│  │  │Tracin│ │      │ │      │ │(AppArmor)  │ │   │
│  │  └──────┘ └──────┘ └──────┘ └────────────┘ │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │          Buildroot Base System                │   │
│  │  Minimal rootfs • systemd • Toolchain        │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

## Layer Details

### 1. Linux Kernel
- **Source**: torvalds/linux (kernel.org)
- **Version**: 6.12 LTS
- **Key configs**:
  - eBPF support for system tracing and monitoring
  - CGroup v2 for resource management
  - Namespace support for sandboxing
  - AppArmor LSM for security
  - KVM guest support for VM deployment
  - Hardware monitoring sensors
  - OverlayFS for container/immutable roots

### 2. Buildroot Base
- Minimal root filesystem tailored for AI workloads
- systemd init for service management
- Python 3.11+ runtime for Hermes Agent
- Node.js runtime for web dashboard
- Essential system tools (htop, ripgrep, jq, etc.)

### 3. Hermes Agent (AI Core)
- **Source**: NousResearch/hermes-agent
- **Role**: Primary AI assistant with full OS integration
- **Capabilities**:
  - Learning loop: creates skills from experience
  - FTS5 memory: cross-session recall with search
  - Autonomous skill creation and improvement
  - Subagent delegation for parallel tasks
  - Cron scheduling for automated operations
  - Multi-platform messaging gateway

### 4. KairosOS Skills (Custom)
Pre-installed skills that give the agent deep system control:

| Skill | Purpose |
|-------|---------|
| System Monitor | CPU/memory/disk/network monitoring with alerts |
| Package Manager | Install/update/remove software |
| Network Manager | Configure interfaces, firewall, DNS |
| Service Manager | systemd service lifecycle management |
| Filesystem Manager | Disk management, permissions, snapshots |
| Security Audit | Vulnerability scanning, hardening |
| Docker Manager | Container lifecycle management |
| Cron Manager | Natural language task scheduling |

### 5. User Interfaces
- **Console TUI**: Full-screen terminal via Hermes Agent CLI
- **Web Dashboard**: Browser-based chat and system monitoring (port 8080)
- **Messaging Gateways**: Telegram, Discord, Slack (optional, via Hermes gateway)

## Data Flow

```
User (any channel)
  │
  ▼
Gateway Layer ───► Hermes Agent ───► Tool Execution
  │                      │                 │
  │                      ▼                 ▼
  │                 Memory Store      Linux System
  │                 (FTS5/SQLite)     (files, procs, net)
  │                      │
  │                      ▼
  │                 Skill Creation
  │                 (autonomous learning)
  │
  └──► Response back to user channel
```

## Security Model

1. **Sandboxed Execution**: Agent runs as unprivileged `kairos` user
2. **Sudo Policy**: Limited to specific system commands
3. **AppArmor**: Kernel-level mandatory access control
4. **DM Pairing**: Gateway requires manual pairing for unknown senders
5. **Private by Default**: No telemetry, no cloud dependency

## Build System

KairosOS uses Buildroot with an external tree:

```
kairosos/
└── buildroot/
    ├── Config.in           # Package selection
    ├── external.mk         # Build integration
    ├── configs/            # Defconfig (kernel + system)
    ├── board/kairosos/     # Rootfs overlay, scripts
    └── package/            # Custom packages
```

Build process:
1. Docker container with build dependencies
2. Buildroot fetches + compiles kernel + toolchain + packages
3. Post-build script installs Hermes Agent + config
4. Post-image script creates ISO + filesystem images
