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
# kairos-ptp-sync - Precision Time Protocol Hardware Sync
case "$ACTION" in
    status|stat) echo "PTP: $(phc_ctl eth0 get 2>/dev/null || echo N/A)"; echo "offset: $(phc2sys -s eth0 -c CLOCK_REALTIME -O 0 -q 2>/dev/null || echo N/A)" ;;
    enable|start|on) ptp4l -i eth0 -f /etc/kairos/ptp.cfg -m 2>/dev/null & echo $! > /var/run/ptp4l.pid; phc2sys -s eth0 -c CLOCK_REALTIME -m 2>/dev/null & echo $! > /var/run/phc2sys.pid; log "PTP sync active" || error "PTP failed" ;;
    disable|stop|off) kill $(cat /var/run/ptp4l.pid 2>/dev/null) 2>/dev/null || true; kill $(cat /var/run/phc2sys.pid 2>/dev/null) 2>/dev/null || true; log "PTP disabled" ;;
    *) usage ;;
esac
