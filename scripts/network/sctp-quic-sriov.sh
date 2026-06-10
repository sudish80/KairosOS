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
# kairos-sctp-quic-sriov - SCTP/QUIC/Multicast/SR-IOV
case "$ACTION" in
    status|stat) lsmod | grep -E "sctp|tls" 2>/dev/null || echo "no sctp/tls"; ip link show type vlan 2>/dev/null | head -3; ls /sys/class/net/*/device/sriov_numvfs 2>/dev/null || echo "no SR-IOV" ;;
    enable|start|on) modprobe sctp 2>/dev/null && log "SCTP loaded" || error "SCTP fail"; modprobe tls 2>/dev/null || true; ip route add 224.0.0.0/4 dev eth0 2>/dev/null || true; log "multicast route added" ;;
    disable|stop|off) modprobe -r sctp 2>/dev/null && log "SCTP unloaded" || true ;;
    sctp) modprobe sctp 2>/dev/null && log "SCTP loaded: $(lsmod | grep sctp | awk "{print \$3}")" || error "SCTP fail" ;;
    quic) socat -u OPENSSL-LISTEN:4433,cert=/etc/kairos/cert.pem,key=/etc/kairos/key.pem,verify=0,fork TCP-LISTEN:4434 2>/dev/null & log "QUIC proxy on 4433" || error "socat fail" ;;
    mcast) ip route add 224.0.0.0/4 dev eth0 2>/dev/null && log "multicast route added" || true ;;
    sriov) for pf in /sys/class/net/*/device/sriov_numvfs; do echo 4 > "$pf" 2>/dev/null && log "4 VFs on $(dirname $(dirname $pf))" || true; done ;;
    *) usage ;;
esac
