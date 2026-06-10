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
# kairos-gpu-power-rail - GPU Power Rail Controller
case "$ACTION" in
    status|stat) for gpu in /sys/class/drm/card*/device/power/control; do echo "$(dirname $(dirname $gpu)): $(cat $gpu 2>/dev/null)"; done; nvidia-smi -q -d POWER 2>/dev/null | head -10 || true ;;
    enable|start|on) for gpu in /sys/bus/pci/devices/*/power/control; do [ -f "$gpu" ] || continue; echo auto > "$gpu" 2>/dev/null && log "GPU power save on $(basename $(dirname $(dirname $gpu)))" || true; done ;;
    disable|stop|off) for gpu in /sys/bus/pci/devices/*/power/control; do [ -f "$gpu" ] || continue; echo on > "$gpu" 2>/dev/null || true; done; log "GPU performance mode" ;;
    *) usage ;;
esac
