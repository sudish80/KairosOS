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
# kairos-port-knock - Port Knocking Daemon
case "$ACTION" in
    status|stat) echo "knockd: $(systemctl is-active knockd 2>/dev/null || echo N/A)"; ls -la /etc/knockd.conf 2>/dev/null || echo "no config" ;;
    enable|start|on) systemctl enable --now knockd 2>/dev/null && log "knockd enabled" || error "knockd not installed"; which knock 2>/dev/null && log "knock client found" || true ;;
    disable|stop|off) systemctl disable --now knockd 2>/dev/null && log "knockd disabled" || true ;;
    *) usage ;;
esac
