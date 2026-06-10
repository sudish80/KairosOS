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
# kairos-display-refresh - Display Panel Refresh Rate
case "$ACTION" in
    status|stat) for card in /sys/class/drm/card*-*; do echo "$(basename $card): status=$(cat $card/status 2>/dev/null) modes=$(cat $card/modes 2>/dev/null | head -1)"; done ;;
    enable|start|on) RATE="${2:-10}"; for card in /sys/class/drm/card*-*; do [ -f "$card/status" ] && [ "$(cat "$card/status")" = "connected" ] || continue; echo "$RATE" > "$card/modes" 2>/dev/null && log "$(basename $card) set to ${RATE}Hz" || true; done ;;
    disable|stop|off) log "display: cannot revert (needs drm reset)" ;;
    *) usage ;;
esac
