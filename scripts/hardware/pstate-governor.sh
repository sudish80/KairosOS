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
# kairos-pstate-governor - CPUFreq Governor Manager
case "$ACTION" in
    status|stat) echo "governor=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null) avail=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors 2>/dev/null)" ;;
    enable|start|on) GOV="${2:-schedutil}"; for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do echo "$GOV" > "$cpu" 2>/dev/null && log "$(basename $(dirname $cpu)) gov=$GOV" || error "set $GOV on $cpu"; done ;;
    disable|stop|off) for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do echo performance > "$cpu" 2>/dev/null || true; done; log "performance governor set" ;;
    *) usage ;;
esac
