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
# kairos-ipsec-vxlan-gps - IPSec/VXLAN/STUN/GPS Quorum
case "$ACTION" in
    status|stat) echo "ipsec: $(ipsec status 2>/dev/null | head -1 || echo down)"; echo "vxlan: $(ip link show vxlan0 2>/dev/null | head -1 || echo down)"; echo "gps: $(cat /sys/class/gps/gps0/qual 2>/dev/null || echo N/A)" ;;
    enable|start|on) ipsec start 2>/dev/null && log "IPSec started" || error "IPSec fail"; ip link add vxlan0 type vxlan id 42 dev eth0 dstport 4789 2>/dev/null && ip link set vxlan0 up && log "VXLAN vxlan0 up" || error "VXLAN fail" ;;
    disable|stop|off) ipsec stop 2>/dev/null && log "IPSec stopped" || true; ip link del vxlan0 2>/dev/null && log "VXLAN removed" || true ;;
    ipsec) ipsec start 2>/dev/null && log "IPSec started" || error "IPSec fail" ;;
    vxlan) ip link add vxlan0 type vxlan id 42 dev eth0 dstport 4789 2>/dev/null && ip link set vxlan0 up && log "VXLAN up" || error "VXLAN fail" ;;
    stun) stun-client "$(curl -s ifconfig.me 2>/dev/null)" 3478 2>/dev/null | head -3 || echo "STUN N/A" ;;
    gps) echo "gps qual: $(cat /sys/class/gps/gps0/qual 2>/dev/null || echo N/A)" ;;
    *) usage ;;
esac
