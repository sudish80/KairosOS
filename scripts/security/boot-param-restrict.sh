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
# kairos-boot-param-restrict - Boot Parameter Restriction
case "$ACTION" in
    status|stat) echo "kptr_restrict=$(cat /proc/sys/kernel/kptr_restrict 2>/dev/null) dmesg_restrict=$(cat /proc/sys/kernel/dmesg_restrict 2>/dev/null)" ;;
    enable|start|on) for p in kptr_restrict dmesg_restrict; do echo 1 > "/proc/sys/kernel/$p" 2>/dev/null && log "$p=1" || error "set $p"; done ;;
    disable|stop|off) for p in kptr_restrict dmesg_restrict; do echo 0 > "/proc/sys/kernel/$p" 2>/dev/null && log "$p=0" || true; done ;;
    *) usage ;;
esac
