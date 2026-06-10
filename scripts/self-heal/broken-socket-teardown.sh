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
# kairos-broken-socket-teardown - Broken Socket Teardown
case "$ACTION" in
    status|stat) ss -tuln 2>/dev/null | head -10 || netstat -tuln 2>/dev/null | head -10 ;;
    enable|start|on) for s in $(ss -tan state CLOSE-WAIT 2>/dev/null | awk "{print \$4}" | grep -v local); do fuser -k "${s}" 2>/dev/null || true; done; log "CLOSE-WAIT sockets cleaned" ;;
    disable|stop|off) log "socket teardown: kernel manages" ;;
    *) usage ;;
esac
