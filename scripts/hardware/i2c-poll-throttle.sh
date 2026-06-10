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
# kairos-i2c-poll-throttle - I2C/SMBus Polling Throttler
case "$ACTION" in
    status|stat) LEVEL=$(cat /sys/class/power_supply/*/capacity 2>/dev/null | head -1 || echo 100); echo "battery: ${LEVEL}%" ;;
    enable|start|on) THRESH="${2:-15}"; LEVEL=$(cat /sys/class/power_supply/*/capacity 2>/dev/null | head -1 || echo 100); if [ "$LEVEL" -lt "$THRESH" ]; then for i2c in /sys/bus/i2c/drivers/*/power/control; do echo auto > "$i2c" 2>/dev/null || true; done; log "I2C throttled (battery ${LEVEL}% < ${THRESH}%)"; else log "I2C normal (battery ${LEVEL}%)"; fi ;;
    disable|stop|off) for i2c in /sys/bus/i2c/drivers/*/power/control; do echo on > "$i2c" 2>/dev/null || true; done; log "I2C full speed" ;;
    *) usage ;;
esac
