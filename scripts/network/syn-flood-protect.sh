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
# kairos-syn-flood-protect - SYN Flood Protection
case "$ACTION" in
    status|stat) echo "tcp_syncookies=$(cat /proc/sys/net/ipv4/tcp_syncookies 2>/dev/null) backlog=$(cat /proc/sys/net/ipv4/tcp_max_syn_backlog 2>/dev/null) syn_retries=$(cat /proc/sys/net/ipv4/tcp_syn_retries 2>/dev/null)" ;;
    enable|start|on) echo 1 > /proc/sys/net/ipv4/tcp_syncookies 2>/dev/null && log "syncookies=1" || error "set fail"; echo 2048 > /proc/sys/net/ipv4/tcp_max_syn_backlog 2>/dev/null || true; echo 2 > /proc/sys/net/ipv4/tcp_syn_retries 2>/dev/null || true ;;
    disable|stop|off) echo 0 > /proc/sys/net/ipv4/tcp_syncookies 2>/dev/null && log "syncookies=0" || true ;;
    *) usage ;;
esac
