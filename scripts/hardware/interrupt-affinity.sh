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
# kairos-interrupt-affinity - IRQ Affinity Steering
case "$ACTION" in
    status|stat) for irq in /proc/irq/[0-9]*/smp_affinity; do echo "$(basename $(dirname $irq)): $(cat $irq 2>/dev/null)"; done | head -20 ;;
    enable|start|on) CORES="${2:-0-3}"; for irq in $(ls /proc/irq/ | grep -E "^[0-9]+$"); do [ -f "/proc/irq/$irq/smp_affinity" ] && echo "$CORES" > "/proc/irq/$irq/smp_affinity" 2>/dev/null || true; done; log "IRQs pinned to cores $CORES" ;;
    disable|stop|off) for irq in $(ls /proc/irq/ | grep -E "^[0-9]+$"); do [ -f "/proc/irq/$irq/smp_affinity" ] && echo f > "/proc/irq/$irq/smp_affinity" 2>/dev/null || true; done; log "IRQ affinity reset" ;;
    *) usage ;;
esac
