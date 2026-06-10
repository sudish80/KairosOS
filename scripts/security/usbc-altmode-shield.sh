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
# kairos-usbc-altmode-shield - USB-C Alt Mode DisplayPort Shield
case "$ACTION" in
    status|stat) for dev in /sys/bus/usb/devices/*; do [ -f "$dev/bInterfaceClass" ] || continue; [ "$(cat "$dev/bInterfaceClass" 2>/dev/null)" = "03" ] || continue; echo "$(basename $dev): $(cat "$dev/authorized" 2>/dev/null)"; done ;;
    enable|start|on) for dev in /sys/bus/usb/devices/*; do [ -f "$dev/bInterfaceClass" ] || continue; [ "$(cat "$dev/bInterfaceClass" 2>/dev/null)" = "03" ] || continue; echo 0 > "$dev/authorized" 2>/dev/null && log "blocked $(basename $dev)" || true; done; log "shield active" ;;
    disable|stop|off) for dev in /sys/bus/usb/devices/*; do [ -f "$dev/bInterfaceClass" ] || continue; [ "$(cat "$dev/bInterfaceClass" 2>/dev/null)" = "03" ] || continue; echo 1 > "$dev/authorized" 2>/dev/null && log "unblocked $(basename $dev)" || true; done ;;
    *) usage ;;
esac
