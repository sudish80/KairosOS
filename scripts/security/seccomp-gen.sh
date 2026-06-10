#!/bin/bash
set -euo pipefail
VERSION="1.0.0"
SCRIPT="${0##*/}"
usage() { echo "Usage: $SCRIPT {status|enable|disable}"; exit 0; }
log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $SCRIPT: $*"; }
error() { log "ERROR: $*" >&2; }
while getopts ":hv" o; do case $o in h) usage;; v) echo "$SCRIPT v$VERSION"; exit 0;; esac; done
shift $((OPTIND-1))
ACTION="${1:-status}"
# kairos-seccomp-gen - Seccomp Profile Generator
case "$ACTION" in
    status|stat) ls -la /etc/kairos/seccomp/ 2>/dev/null || echo "no seccomp profiles"; cat /proc/$$/status | grep Seccomp || true ;;
    enable|start|on) mkdir -p /etc/kairos/seccomp; for bin in kairos-apply kairos-mcp kairos-bpf kairos-inference-hub; do which "$bin" 2>/dev/null || continue; strace -c -o "/tmp/${bin}.strace" "$bin" --help 2>/dev/null || true; done; log "binaries profiled" ;;
    disable|stop|off) echo "seccomp profiles remain" ;;
    *) usage ;;
esac
