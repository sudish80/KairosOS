#!/bin/bash
# kairos-agent-tool-executor - Hermes-invokable hardware/network/security tools
set -euo pipefail
VERSION="1.0.0"
SCRIPT="${0##*/}"
TOOL_DIR="/usr/lib/kairos/tools"
LOG_DIR="/var/log/kairos/tools"
log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $SCRIPT: $*"; }
error() { log "ERROR: $*" >&2; }
usage() {
    cat <<EOF
Usage: $SCRIPT <tool> [args...]
Tools: hardware: interrupt-affinity, pcie-aspm, cpu-core-park, nvme-apst, thp-defrag, cpu-cstate-latency, edac-monitor, pstate-governor
       security: panic-switch, seccomp-gen, typing-cadence-pam, mac-spatial-shifter, boot-param-restrict, null-ptr-guard
       network: dns-leak-eliminator, port-knock, bbr-switcher, syn-flood-protect, wireguard-mesh
       self-heal: orphan-process-reaper, broken-socket-teardown, broken-service-backoff, symlink-repair, smart-predict
       storage: io-scheduler-hotswap, zswap-tuner, nvme-apst
EOF
    exit 0
}
mkdir -p "$LOG_DIR"
if [ $# -eq 0 ]; then usage; fi
TOOL="$1"; shift
if [ -x "$TOOL_DIR/$TOOL" ]; then
    "$TOOL_DIR/$TOOL" "$@" 2>&1 | tee -a "$LOG_DIR/$TOOL.log"
else
    error "tool not found: $TOOL (not in $TOOL_DIR)"
    usage
fi
