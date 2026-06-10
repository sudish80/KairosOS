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
# kairos-usb-descriptor-strip - USB Descriptor Stripping
case "$ACTION" in
    status|stat) for dev in /sys/bus/usb/devices/*/authorized; do echo "$(basename $(dirname $dev)): $(cat $dev 2>/dev/null)"; done ;;
    enable|start|on) for dev in /sys/bus/usb/devices/*/bInterfaceClass; do bc=$(cat "$dev" 2>/dev/null); [ -z "$bc" ] && continue; echo 0 > "$(dirname $dev)/authorized" 2>/dev/null || true; done; log "non-essential USB deauthorized" ;;
    disable|stop|off) for dev in /sys/bus/usb/devices/*/authorized; do echo 1 > "$dev" 2>/dev/null || true; done; log "all USB re-authorized" ;;
    *) usage ;;
esac
