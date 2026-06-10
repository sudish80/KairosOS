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
# kairos-irq-blackhole - Interrupt Vector Blackhole Redirect
case "$ACTION" in
    status|stat) echo "IRQ blackhole: check /proc/irq/99/smp_affinity" ;;
    enable|start|on) IRQ="${2:-99}"; for irq in /proc/irq/*; do n=$(basename "$irq"); [ "$n" -eq "$IRQ" ] 2>/dev/null || continue; echo 0 > "$irq/smp_affinity" 2>/dev/null && log "IRQ $IRQ blackholed" || error "failed"; done ;;
    disable|stop|off) IRQ="${2:-99}"; for irq in /proc/irq/*; do n=$(basename "$irq"); [ "$n" -eq "$IRQ" ] 2>/dev/null || continue; echo f > "$irq/smp_affinity" 2>/dev/null && log "IRQ $IRQ restored" || true; done ;;
    *) usage ;;
esac
