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
# kairos-dns-leak-eliminator - DNS Leak Eliminator
case "$ACTION" in
    status|stat) echo "resolv.conf:"; cat /etc/resolv.conf 2>/dev/null; echo "VPN DNS: $(nmcli -t -f NAME con show --active 2>/dev/null | head -1)" ;;
    enable|start|on) mkdir -p /etc/NetworkManager/conf.d; echo -e "[main]\ndns=default\n" > /etc/NetworkManager/conf.d/dns-leak.conf 2>/dev/null && log "DNS leak protection enabled" || error "config fail" ;;
    disable|stop|off) rm -f /etc/NetworkManager/conf.d/dns-leak.conf 2>/dev/null && log "DNS leak protection disabled" || true ;;
    *) usage ;;
esac
