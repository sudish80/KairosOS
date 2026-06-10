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
# kairos-wireguard-mesh - WireGuard Mesh Networking
case "$ACTION" in
    status|stat) echo "wg: $(wg show 2>/dev/null | head -5 || echo "no interfaces")"; ip link show kmesh0 2>/dev/null | head -1 || echo "kmesh0 down" ;;
    enable|start|on) [ -f /etc/kairos/wg/key ] || (wg genkey | tee /etc/kairos/wg/key | wg pubkey > /etc/kairos/wg/key.pub) 2>/dev/null; wg-quick up kmesh0 2>/dev/null && log "kmesh0 up" || error "wg-quick fail" ;;
    disable|stop|off) wg-quick down kmesh0 2>/dev/null && log "kmesh0 down" || true ;;
    *) usage ;;
esac
