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
# kairos-io-scheduler-hotswap - I/O Scheduler Hotswap
case "$ACTION" in
    status|stat) for dev in /sys/block/*/queue/scheduler; do echo "$(basename $(dirname $dev)): $(cat $dev 2>/dev/null)"; done ;;
    enable|start|on) SCHED="${2:-none}"; for dev in /sys/block/*/queue/scheduler; do echo "$SCHED" > "$dev" 2>/dev/null && log "$(basename $(dirname $dev)) sched=$SCHED" || error "$(basename $(dirname $dev)) set fail"; done ;;
    disable|stop|off) for dev in /sys/block/*/queue/scheduler; do echo mq-deadline > "$dev" 2>/dev/null || true; done; log "scheduler reset to mq-deadline" ;;
    *) usage ;;
esac
