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
# kairos-fan-curve-synth - Fan Curve Synthesizer
case "$ACTION" in
    status|stat) for hwmon in /sys/class/hwmon/hwmon*; do [ -f "$hwmon/fan1_input" ] && echo "$(basename $hwmon): fan=$(cat $hwmon/fan1_input 2>/dev/null) temp=$(cat $hwmon/temp1_input 2>/dev/null)"; done ;;
    enable|start|on) log "fan curve kernel-managed (use fan-rpm-gov.sh for PWM control)" ;;
    disable|stop|off) echo "fan curve: kernel-managed" ;;
    *) usage ;;
esac
