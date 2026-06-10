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
# kairos-mptcp-vlan-bridge - MPTCP/VLAN Bridge
case "$ACTION" in
    status|stat) echo "mptcp: $(sysctl -n net.mptcp.enabled 2>/dev/null || echo N/A)"; ip link show type vlan 2>/dev/null | head -5 || echo "no vlans" ;;
    enable|start|on) echo 1 > /proc/sys/net/mptcp/enabled 2>/dev/null && log "MPTCP enabled" || error "MPTCP N/A"; ip link add link eth0 name eth0.100 type vlan id 100 2>/dev/null && ip link set eth0.100 up && log "VLAN 100 up" || true ;;
    disable|stop|off) echo 0 > /proc/sys/net/mptcp/enabled 2>/dev/null && log "MPTCP disabled" || true; ip link del eth0.100 2>/dev/null && log "VLAN 100 removed" || true ;;
    *) usage ;;
esac
