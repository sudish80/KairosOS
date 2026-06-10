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
# kairos-pcie-aspm - Dynamic PCIe Link State Management
case "$ACTION" in
    status|stat) for dev in /sys/bus/pci/devices/*/power/control; do echo "$(basename $(dirname $dev)): $(cat $dev 2>/dev/null)"; done ;;
    enable|start|on) for dev in /sys/bus/pci/devices/*/power/control; do echo auto > "$dev" 2>/dev/null; done; log "PCIe ASPM powersave" ;;
    disable|stop|off) for dev in /sys/bus/pci/devices/*/power/control; do echo on > "$dev" 2>/dev/null; done; log "PCIe ASPM performance" ;;
    *) usage ;;
esac
