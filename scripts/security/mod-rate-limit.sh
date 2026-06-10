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
# kairos-mod-rate-limit - Kernel Module Loading Rate Limiter
case "$ACTION" in
    status|stat) echo "max_loads=$(cat /proc/sys/kernel/modules_max_loads 2>/dev/null) loaded=$(lsmod | wc -l)" ;;
    enable|start|on) echo 5 > /proc/sys/kernel/modules_max_loads 2>/dev/null && log "mod rate limit 5/sec" || error "N/A" ;;
    disable|stop|off) echo 0 > /proc/sys/kernel/modules_max_loads 2>/dev/null && log "mod rate limit off" || true ;;
    *) usage ;;
esac
