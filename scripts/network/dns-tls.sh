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
# kairos-dns-tls - DNS-over-TLS Configuration
case "$ACTION" in
    status|stat) echo "DNS: $(cat /etc/resolv.conf 2>/dev/null | grep -i 'nameserver')"; ls -la /etc/stubby/stubby.yml 2>/dev/null || echo "no stubby config" ;;
    enable|start|on) echo -e "[Resolve]\nDNS=1.1.1.1 9.9.9.9\nDNSOverTLS=yes\n" > /etc/systemd/resolved.conf.d/dns-over-tls.conf 2>/dev/null && systemctl restart systemd-resolved 2>/dev/null && log "DNS-over-TLS enabled" || error "config fail" ;;
    disable|stop|off) rm -f /etc/systemd/resolved.conf.d/dns-over-tls.conf 2>/dev/null && systemctl restart systemd-resolved 2>/dev/null && log "DNS-over-TLS disabled" || true ;;
    *) usage ;;
esac
