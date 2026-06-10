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
# kairos-initramfs-sign - Boot Chain Initramfs Signing
case "$ACTION" in
    status|stat) ls -la /boot/*.sig 2>/dev/null || echo "no sigs"; ls -la /etc/kairos/boot.key 2>/dev/null || echo "no key" ;;
    enable|start|on) for img in /boot/initramfs-*; do [ -f "$img" ] || continue; openssl dgst -sha256 -sign /etc/kairos/boot.key -out "$img.sig" "$img" 2>/dev/null && log "signed $(basename $img)" || error "sign fail $img"; done ;;
    disable|stop|off) for img in /boot/initramfs-*; do [ -f "$img.sig" ] || continue; openssl dgst -sha256 -verify /etc/kairos/boot.pub -signature "$img.sig" "$img" 2>/dev/null && log "verified $(basename $img)" || error "FAIL $img"; done ;;
    *) usage ;;
esac
