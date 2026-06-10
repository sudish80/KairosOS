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
# kairos-reboot-gov - Software-Controlled Reboot Logic
case "$ACTION" in
    status|stat) uptime ;;
    enable|start|on) log "soft reset: restarting kairos services"; for svc in /etc/systemd/system/kairos-*.service; do systemctl restart "$(basename $svc)" 2>/dev/null || true; done ;;
    disable|stop|off) log "powercycle: forcing reboot"; /sbin/reboot -f 2>/dev/null || error "reboot failed" ;;
    bmc) ipmitool mc reset warm 2>/dev/null && log "BMC reset" || error "IPMI N/A" ;;
    *) usage ;;
esac
