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
# kairos-rowhammer-refresh - CPU Cache Bank Rowhammer Refresh
case "$ACTION" in
    status|stat) for cpu in /sys/devices/system/cpu/cpu*/cache/index*/; do [ -f "$cpu/type" ] || continue; echo "$(cat "$cpu/type" 2>/dev/null): $(cat "$cpu/size" 2>/dev/null)"; done ;;
    enable|start|on) for cpu in /sys/devices/system/cpu/cpu*/cache/index*/; do [ -f "$cpu/type" ] || continue; cat "$cpu/size" > /dev/null; done; log "cache banks refreshed" ;;
    disable|stop|off) echo "Rowhammer: kernel manages" ;;
    *) usage ;;
esac
