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
# kairos-cpu-cstate-latency - CPU C-State Latency Overrider
case "$ACTION" in
    status|stat) for cpu in /sys/devices/system/cpu/cpu*/cpuidle; do for s in "$cpu"/state*; do [ -f "$s/name" ] && echo "$(basename $(dirname $s))/$(basename $s): $(cat $s/name 2>/dev/null) lat=$(cat $s/latency 2>/dev/null)us"; done; break; done ;;
    enable|start|on) LAT="${2:-0}"; echo "$LAT" > /dev/cpu_dma_latency 2>/dev/null && log "C-state lat locked to ${LAT}us" || error "PM_QOS not available" ;;
    disable|stop|off) echo "1000000000" > /dev/cpu_dma_latency 2>/dev/null && log "C-state latency unlocked" || true ;;
    *) usage ;;
esac
