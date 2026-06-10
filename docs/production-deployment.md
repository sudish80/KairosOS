# KairosOS Production Deployment Guide

## 1. System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | x86_64 or aarch64, 2 cores | 8+ cores |
| RAM | 4 GB | 16+ GB |
| Storage | 32 GB SSD/NVMe | 256 GB+ NVMe |
| TPM | TPM 2.0 | TPM 2.0 + discrete |
| Network | 1× GbE | 2× 10GbE |

## 2. Installation

### Option A: Debian/Ubuntu

```bash
# Add repository
curl -fsSL https://packages.kairosos.org/gpg | gpg --dearmor | sudo tee /usr/share/keyrings/kairosos.gpg
echo "deb [signed-by=/usr/share/keyrings/kairosos.gpg] https://packages.kairosos.org/apt stable main" | \
    sudo tee /etc/apt/sources.list.d/kairosos.list

# Install
sudo apt update
sudo apt install kairosos-full
```

### Option B: Fedora/RHEL

```bash
sudo dnf install dnf-plugins-core
sudo dnf config-manager --add-repo https://packages.kairosos.org/rpm/kairosos.repo
sudo dnf install kairosos-daemons kairosos-kernel-modules
```

### Option C: Arch Linux

```bash
yay -S kairosos
```

### Option D: ISO (Buildroot)

```bash
sudo make build-iso CONFIG=kairosos_defconfig
```

### Option E: From Source

```bash
sudo apt install cargo rustc python3 python3-pip linux-headers-$(uname -r)
git clone https://github.com/kairosos/kairosos.git
cd kairosos
sudo make install
```

## 3. Post-Installation

### Enable Services

```bash
# Core daemons
sudo systemctl enable --now kairos-bpf kairos-mcp kairos-apply
sudo systemctl enable --now kairos-recovery kairos-git-logger

# Optional services
sudo systemctl enable --now kairos-orchestrator kairos-mesh kairos-db
sudo systemctl enable --now kairos-tui kairos-fb

# AI microservices
sudo systemctl enable --now kairos-confidence kairos-supervisor
sudo systemctl enable --now kairos-hermes-agent kairos-knowledge-graph

# OTA update scheduler
sudo systemctl enable --now kairos-update-check.timer
```

### Load Kernel Modules

```bash
sudo modprobe kairos_dmverity kairos_tpm kairos_edac
sudo modprobe kairos_ptp kairos_fb kairos_iommu kairos_prochot

# Auto-load on boot
echo "kairos_dmverity kairos_tpm kairos_edac kairos_ptp kairos_fb kairos_iommu kairos_prochot" | \
    sudo tee /etc/modules-load.d/kairosos.conf
```

### Apply AppArmor Profiles

```bash
sudo systemctl enable --now apparmor
sudo apparmor_parser -r /etc/apparmor.d/kairos-*
sudo aa-enforce /etc/apparmor.d/kairos-*
```

### Configure OTA Updates

Edit `/etc/kairos/recovery.toml`:

```toml
[update]
server_url = "https://updates.kairosos.org/v1"
channel = "stable"
auto_check = true
auto_apply = false
staging_percentage = 10
```

## 4. OTA Update Workflow

### Manual

```bash
# Check for updates
sudo ota-update.sh check

# Download update
sudo ota-update.sh download

# Apply update
sudo ota-update.sh apply
sudo reboot

# Rollback if needed
sudo ota-update.sh rollback
sudo reboot
```

### Automatic (Scheduled)

The systemd timer `kairos-update-check.timer` checks daily with randomized delay.
Set `auto_apply = true` in `recovery.toml` for fully automatic updates.

## 5. Monitoring

### Health Checks

```bash
# Manual health check
kairos-recovery --health

# View health metrics (JSON)
curl --unix-socket /var/run/kairos/mcp.sock http://localhost/mcp \
    -d '{"jsonrpc":"2.0","method":"get_health","id":1}'
```

### Telemetry

```bash
# System telemetry
kairos-recovery --metrics

# AI service telemetry
echo '{"jsonrpc":"2.0","method":"stats","params":{"name":"cpu_temp"},"id":1}' | \
    socat - UNIX-CONNECT:/var/run/kairos/telemetry-collector.sock
```

### Logs

```bash
# All KairosOS daemon logs
journalctl -t kairos -f

# Specific daemon
journalctl -u kairos-bpf -f

# Update history
journalctl -u kairos-recovery _COMM=kairos-recovery
```

## 6. Security

### AppArmor Enforcement

All 22 daemons have dedicated AppArmor profiles. Verify enforcement:

```bash
sudo aa-status | grep kairos
```

Expected: all `kairos-*` profiles in `enforce` mode.

### Secure Boot

```bash
sudo scripts/security/pk-enroll.sh
sudo scripts/security/initramfs-sign.sh
```

### TPM Integration

```bash
sudo kairos-tpm --status    # View PCR values
sudo kairos-tpm --extend    # Extend PCR with system state
```

### Integrity Verification

```bash
sudo dmverity --status      # Check dm-verity status
sudo kairos-recovery --health  # Full system health check
```

## 7. Backup & Recovery

### A/B Slots

The system maintains two root partitions (`KAIROS_A`, `KAIROS_B`).
Active slot auto-fallback: 3 failed boots → automatic rollback.

### Manual Recovery

```bash
# Enter recovery shell
kairos-recovery --shell

# Manual slot switch
kairos-recovery --switch b

# Force rollback
kairos-recovery --rollback
```

### Configuration Backup

All system config is tracked in `/var/lib/kairos/git/state.git`:

```bash
cd /var/lib/kairos/git/state.git
git log --oneline
git diff HEAD~1
```

## 8. Production Checklist

- [ ] All 22 AppArmor profiles loaded in enforce mode
- [ ] Kernel modules loaded (`lsmod | grep kairos`)
- [ ] Core systemd services enabled and running
- [ ] OTA update channel configured
- [ ] `/etc/kairos/gpg` keyring populated with update signing keys
- [ ] TPM 2.0 enabled and readable (`sudo kairos-tpm --status`)
- [ ] dm-verity root hash configured
- [ ] Network time sync active (PTP or NTS)
- [ ] Monitoring/dashboard configured
- [ ] Backup plan documented

## 9. Architecture Diagram

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

## 10. Troubleshooting

### Daemon won't start

```bash
journalctl -u kairos-bpf -n 50 --no-pager
# Check AppArmor: sudo aa-status | grep kairos-bpf
# Check config: /etc/kairos/bpf.toml
```

### Update fails

```bash
ota-update.sh status
journalctl -u kairos-recovery -n 50
# Check /var/lib/kairos/updates/ for partial downloads
# Verify GPG keyring: gpgv --keyring /etc/kairos/gpg
```

### Kernel module not loading

```bash
dmesg | grep kairos
# Check kernel config: zgrep CONFIG_BPF /proc/config.gz
# Verify module: modinfo kairos_dmverity
```
