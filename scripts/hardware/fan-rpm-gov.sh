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
# kairos-fan-rpm-gov - Thermal Zone Fan RPM Governor
case "$ACTION" in
    status|stat) for hwmon in /sys/class/hwmon/hwmon*; do [ -f "$hwmon/fan1_input" ] && echo "$(basename $hwmon): temp=$(cat $hwmon/temp1_input 2>/dev/null) fan=$(cat $hwmon/fan1_input 2>/dev/null) pwm=$(cat $hwmon/pwm1 2>/dev/null)"; done ;;
    enable|start|on) THRESH="${2:-70}"; for hwmon in /sys/class/hwmon/hwmon*; do [ -f "$hwmon/pwm1" ] || continue; temp=$(( $(cat "$hwmon/temp1_input" 2>/dev/null || echo 0) / 1000 )); [ "$temp" -gt "$THRESH" ] && pwm=$(( 128 + (temp - THRESH) * 127 / 30 )) || pwm=$(( temp * 128 / THRESH )); [ "$pwm" -gt 255 ] && pwm=255; echo "$pwm" > "$hwmon/pwm1" 2>/dev/null && log "$(basename $hwmon) pwm=$pwm at ${temp}C" || true; done ;;
    disable|stop|off) for hwmon in /sys/class/hwmon/hwmon*; do [ -f "$hwmon/pwm1" ] && echo 255 > "$hwmon/pwm1" 2>/dev/null; done; log "fans set to max" ;;
    *) usage ;;
esac
