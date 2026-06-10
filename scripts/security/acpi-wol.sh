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
# kairos-acpi-wol - ACPI Sleep/Wake-On-LAN Controller
case "$ACTION" in
    status|stat) echo "sleep_state=$(cat /sys/power/state 2>/dev/null)"; for iface in /sys/class/net/*; do ethtool $(basename $iface) 2>/dev/null | grep Wake-on || true; done ;;
    enable|start|on) echo N > /sys/module/acpi/parameters/sleep_state 2>/dev/null && log "S3 blocked" || error "no sysfs"; for iface in /sys/class/net/*; do ethtool -s "$(basename $iface)" wol g 2>/dev/null && true; done ;;
    disable|stop|off) for iface in /sys/class/net/*; do ethtool -s "$(basename $iface)" wol d 2>/dev/null || true; done; log "WOL disabled" ;;
    *) usage ;;
esac
