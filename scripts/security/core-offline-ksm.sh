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
# kairos-core-offline-ksm - Core Offline KSM
case "$ACTION" in
    status|stat) echo "ksm_run=$(cat /sys/kernel/mm/ksm/run 2>/dev/null) pages_shared=$(cat /sys/kernel/mm/ksm/pages_shared 2>/dev/null)" ;;
    enable|start|on) echo 1 > /sys/kernel/mm/ksm/run 2>/dev/null && log "KSM enabled" || error "KSM N/A"; echo 100 > /sys/kernel/mm/ksm/pages_to_scan 2>/dev/null || true ;;
    disable|stop|off) echo 0 > /sys/kernel/mm/ksm/run 2>/dev/null && log "KSM disabled" || true ;;
    *) usage ;;
esac
