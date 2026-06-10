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
# kairos-orphan-process-reaper - Orphan Process Reaper
case "$ACTION" in
    status|stat) ps -eo pid,ppid,state,comm --no-headers 2>/dev/null | awk "\$2 == 1 && \$3 ~ /[Z]/" | head -10 || echo "no zombie orphans" ;;
    enable|start|on) for p in $(ps -eo pid,ppid,state --no-headers 2>/dev/null | awk "\$2 == 1 && \$3 ~ /[Z]/ {print \$1}"); do kill -9 "$p" 2>/dev/null && log "reaped orphan $p" || true; done ;;
    disable|stop|off) log "orphan reaper: kernel manages (init reaps)" ;;
    *) usage ;;
esac
