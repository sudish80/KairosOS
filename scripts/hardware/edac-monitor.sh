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
# kairos-edac-monitor - ECC/EDAC Monitor
case "$ACTION" in
    status|stat) if [ -d /sys/devices/system/edac ]; then for mc in /sys/devices/system/edac/mc*; do echo "$(basename $mc): CE=$(cat $mc/ce_count 2>/dev/null) UE=$(cat $mc/ue_count 2>/dev/null)"; done; else echo "EDAC N/A"; fi; for z in /sys/class/thermal/thermal_zone*; do [ -f "$z/type" ] && grep -qi "dimm\|memory\|dram" "$z/type" 2>/dev/null && echo "thermal: $(cat $z/type) $(cat $z/temp 2>/dev/null)C"; done ;;
    enable|start|on) log "EDAC: kernel-managed error tracking" ;;
    disable|stop|off) echo "EDAC: cannot disable" ;;
    inject) echo "HWPoison inject requires kernel CONFIG_HWPOISON_INJECT" ;;
    *) usage ;;
esac
