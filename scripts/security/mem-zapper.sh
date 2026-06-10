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
# kairos-mem-zapper - Memory Region Zapping Pre-allocator
case "$ACTION" in
    status|stat) ls -la /dev/shm/zapped 2>/dev/null || echo "no zapped region" ;;
    enable|start|on) SIZE="${2:-64}"; dd if=/dev/zero of=/dev/shm/zapped bs=1M count=$SIZE 2>/dev/null && log "zapped ${SIZE}MB" || error "zap fail" ;;
    disable|stop|off) rm -f /dev/shm/zapped && log "region freed" || true ;;
    *) usage ;;
esac
