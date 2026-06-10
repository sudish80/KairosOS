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
# kairos-thp-defrag - THP Defragmentation Governor
case "$ACTION" in
    status|stat) echo "THP: $(cat /sys/kernel/mm/transparent_hugepage/enabled 2>/dev/null) defrag=$(cat /sys/kernel/mm/transparent_hugepage/defrag 2>/dev/null)" ;;
    enable|start|on) echo 1 > /proc/sys/vm/compact_memory 2>/dev/null && log "memory compaction triggered"; echo always > /sys/kernel/mm/transparent_hugepage/defrag 2>/dev/null && log "THP defrag=always" || true ;;
    disable|stop|off) echo defer > /sys/kernel/mm/transparent_hugepage/defrag 2>/dev/null && log "THP defrag=defer" || true ;;
    compact) echo 1 > /proc/sys/vm/compact_memory 2>/dev/null && log "memory compaction" || error "compact N/A" ;;
    *) usage ;;
esac
