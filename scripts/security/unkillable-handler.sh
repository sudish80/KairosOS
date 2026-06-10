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
# kairos-unkillable-handler - Unkillable Process Abort Handler
case "$ACTION" in
    status|stat) echo "D-state procs:"; ps -eo pid,state,comm 2>/dev/null | awk "/\s+[0-9]+\s+D\s+/" || echo "none" ;;
    enable|start|on) for p in $(ps -eo pid,state | awk "/D/ {print \$1}" 2>/dev/null); do kill -9 "$p" 2>/dev/null || true; done; log "D-state procs killed" ;;
    disable|stop|off) echo "stateless: no disable" ;;
    *) usage ;;
esac
