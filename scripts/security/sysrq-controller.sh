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
# kairos-sysrq-controller - Magic SysRq Access Controller
case "$ACTION" in
    status|stat) echo "SysRq=$(cat /proc/sys/kernel/sysrq 2>/dev/null)" ;;
    enable|start|on) echo 1 > /proc/sys/kernel/sysrq 2>/dev/null && log "SysRq full" || error "set fail" ;;
    disable|stop|off) echo 0 > /proc/sys/kernel/sysrq 2>/dev/null && log "SysRq off" || error "set fail"; echo 4 > /proc/sys/kernel/sysrq 2>/dev/null && log "SysRq restricted" || true ;;
    *) usage ;;
esac
