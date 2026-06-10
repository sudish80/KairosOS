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
# kairos-sata-lpm - SATA Link Power Management
case "$ACTION" in
    status|stat) for host in /sys/class/scsi_host/host*/link_power_management_policy; do echo "$(basename $(dirname $host)): $(cat $host 2>/dev/null)"; done ;;
    enable|start|on) for host in /sys/class/scsi_host/host*/link_power_management_policy; do echo min_power > "$host" 2>/dev/null && log "SATA LPM min_power on $(basename $(dirname $host))" || true; done ;;
    disable|stop|off) for host in /sys/class/scsi_host/host*/link_power_management_policy; do echo max_performance > "$host" 2>/dev/null && log "SATA max_performance on $(basename $(dirname $host))" || true; done ;;
    *) usage ;;
esac
