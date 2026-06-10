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
# kairos-broken-service-backoff - Broken Service Backoff
case "$ACTION" in
    status|stat) systemctl list-units --type=service --state=failed --no-pager --no-legend 2>/dev/null | head -10 || echo "no failed services" ;;
    enable|start|on) for svc in $(systemctl list-units --type=service --state=failed --no-legend 2>/dev/null | awk "{print \$1}"); do systemctl reset-failed "$svc" 2>/dev/null && log "reset $svc" || error "reset $svc"; done ;;
    disable|stop|off) log "auto: systemd handles service backoff" ;;
    *) usage ;;
esac
