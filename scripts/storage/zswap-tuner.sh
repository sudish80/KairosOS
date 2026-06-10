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
# kairos-zswap-tuner - ZSwap Compressed Cache Tuner
case "$ACTION" in
    status|stat) echo "zswap: $(cat /sys/module/zswap/parameters/enabled 2>/dev/null) compressor=$(cat /sys/module/zswap/parameters/compressor 2>/dev/null) max_pool=$(cat /sys/module/zswap/parameters/max_pool_percent 2>/dev/null)" ;;
    enable|start|on) echo Y > /sys/module/zswap/parameters/enabled 2>/dev/null && log "zswap enabled" || error "zswap N/A (module loaded?)"; echo lz4 > /sys/module/zswap/parameters/compressor 2>/dev/null || true ;;
    disable|stop|off) echo N > /sys/module/zswap/parameters/enabled 2>/dev/null && log "zswap disabled" || true ;;
    *) usage ;;
esac
