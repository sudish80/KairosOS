#!/bin/bash
# kairos-first-boot - First-boot initialization sequence
set -euo pipefail
VERSION="1.0.0"
SCRIPT="${0##*/}"
log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $SCRIPT: $*"; }
error() { log "ERROR: $*" >&2; }
log "KairosOS first-boot v$VERSION starting"
mkdir -p /var/lib/kairos /var/run/kairos /etc/kairos/generations /var/log/kairos /tmp/kairos
for svc in kairos-apply kairos-bpf kairos-git-logger kairos-inference-hub kairos-mcp kairos-recovery; do
    if systemctl is-enabled "$svc" 2>/dev/null | grep -q enabled; then
        systemctl start "$svc" 2>/dev/null && log "$svc started" || error "$svc start failed"
    fi
done
if [ -f /etc/kairos/configuration.nix ]; then
    kairos-apply --apply /etc/kairos/configuration.nix 2>/dev/null && log "initial config applied" || error "apply failed"
fi
find /etc/kairos -name "*.toml" -newer /etc/kairos -exec kairos-git-logger commit {} \; 2>/dev/null || true
log "First-boot complete"
