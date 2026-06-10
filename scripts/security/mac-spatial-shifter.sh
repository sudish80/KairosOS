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
# kairos-mac-spatial-shifter - Dynamic MAC Address Spatial Shifter
case "$ACTION" in
    status|stat) IFACE="${2:-wlan0}"; echo "$IFACE: $(ip link show $IFACE 2>/dev/null | grep -o 'ether [0-9a-f:]*' | cut -d' ' -f2)"; echo "perm: $(ethtool -P $IFACE 2>/dev/null | awk "{print \$3}")" ;;
    enable|start|on) IFACE="${2:-wlan0}"; ip link set "$IFACE" down 2>/dev/null || error "cannot down $IFACE"; NEW_MAC=$(hexdump -n 6 -e '6/1 "%02X"' /dev/urandom 2>/dev/null | sed 's/^/\$(date +%S | md5sum | head -c 2)/' | head -c 12 | sed 's/\(..\)/:\1/g' | sed 's/^://'); ip link set "$IFACE" address "$NEW_MAC" 2>/dev/null && log "$IFACE MAC randomized to $NEW_MAC" || error "MAC set fail"; ip link set "$IFACE" up 2>/dev/null || true ;;
    disable|stop|off) IFACE="${2:-wlan0}"; ip link set "$IFACE" down 2>/dev/null || true; ORIG=$(ethtool -P "$IFACE" 2>/dev/null | awk "{print \$3}"); ip link set "$IFACE" address "$ORIG" 2>/dev/null && log "$IFACE MAC reset" || error "reset fail"; ip link set "$IFACE" up 2>/dev/null || true ;;
    *) usage ;;
esac
