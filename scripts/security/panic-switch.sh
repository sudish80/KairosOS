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
# kairos-panic-switch - Kernel Panic Switch
case "$ACTION" in
    status|stat) echo "panic=$(cat /proc/sys/kernel/panic 2>/dev/null) panic_on_oops=$(cat /proc/sys/kernel/panic_on_oops 2>/dev/null) softlockup=$(cat /proc/sys/kernel/softlockup_panic 2>/dev/null)" ;;
    enable|start|on) echo 10 > /proc/sys/kernel/panic 2>/dev/null && log "panic timeout 10s" || error "set fail"; echo 1 > /proc/sys/kernel/panic_on_oops 2>/dev/null || true; echo 1 > /proc/sys/kernel/softlockup_panic 2>/dev/null || true ;;
    disable|stop|off) echo 0 > /proc/sys/kernel/panic 2>/dev/null && log "panic=0 (reboot)" || true ;;
    *) usage ;;
esac
