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
# kairos-bbr-switcher - TCP BBR Congestion Control Switcher
case "$ACTION" in
    status|stat) echo "CC: $(sysctl -n net.ipv4.tcp_congestion_control 2>/dev/null) avail: $(sysctl -n net.ipv4.tcp_available_congestion_control 2>/dev/null)" ;;
    enable|start|on) echo bbr > /proc/sys/net/ipv4/tcp_congestion_control 2>/dev/null && log "BBR enabled" || error "BBR not available" ;;
    disable|stop|off) echo cubic > /proc/sys/net/ipv4/tcp_congestion_control 2>/dev/null && log "Cubic restored" || error "set fail" ;;
    cubic) echo cubic > /proc/sys/net/ipv4/tcp_congestion_control 2>/dev/null && log "Cubic set" || error "set fail" ;;
    *) usage ;;
esac
