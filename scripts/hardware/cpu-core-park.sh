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
# kairos-cpu-core-park - CPU Core Parking Optimizer
case "$ACTION" in
    status|stat) for cpu in /sys/devices/system/cpu/cpu*/online; do echo "$(basename $(dirname $cpu)): $(cat $cpu 2>/dev/null)"; done ;;
    enable|start|on) RANGE="${2:-4-7}"; for cpu in /sys/devices/system/cpu/cpu*/online; do c=$(basename $(dirname "$cpu") | sed 's/cpu//'); [[ "$c" =~ ^[0-9]+$ ]] || continue; [ "$c" -ge "${RANGE%-*}" ] 2>/dev/null && [ "$c" -le "${RANGE#*-}" ] 2>/dev/null && echo 0 > "$cpu" 2>/dev/null && log "cpu$c parked" || true; done ;;
    disable|stop|off) for cpu in /sys/devices/system/cpu/cpu*/online; do echo 1 > "$cpu" 2>/dev/null || true; done; log "all cores unparked" ;;
    *) usage ;;
esac
