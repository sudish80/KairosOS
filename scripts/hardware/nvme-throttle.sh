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
# kairos-nvme-throttle - NVMe Thermal Throttle Override
case "$ACTION" in
    status|stat) for nvme in /sys/class/nvme/nvme*; do echo "$(basename $nvme): throttle=$(cat $nvme/device/throttle_temp 2>/dev/null) temp=$(cat $nvme/device/temp1 2>/dev/null)"; done ;;
    enable|start|on) THR="${2:-75}"; for nvme in /sys/class/nvme/nvme*; do [ -f "$nvme/device/throttle_temp" ] && echo "$((THR*1000))" > "$nvme/device/throttle_temp" 2>/dev/null && log "$(basename $nvme) throttle set to ${THR}C" || true; done ;;
    disable|stop|off) for nvme in /sys/class/nvme/nvme*; do [ -f "$nvme/device/throttle_temp" ] && echo "0" > "$nvme/device/throttle_temp" 2>/dev/null || true; done; log "NVMe throttle disabled" ;;
    *) usage ;;
esac
