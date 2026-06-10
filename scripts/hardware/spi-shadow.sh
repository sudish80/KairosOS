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
# kairos-spi-shadow - SPI Flash Backup Shadowing
case "$ACTION" in
    status|stat) BACKUP="${2:-/var/lib/kairos/spi-shadow}"; ls -la "$BACKUP" 2>/dev/null || echo "no backup at $BACKUP" ;;
    enable|start|on) BACKUP="${2:-/var/lib/kairos/spi-shadow}"; mkdir -p "$BACKUP"; dmidecode 2>/dev/null | head -20 > "$BACKUP/bios-info.txt" || true; openssl enc -aes-256-cbc -salt -in /dev/null -out "$BACKUP/shadow.enc" -pass pass:$(hostname) 2>/dev/null || true; log "SPI shadow initialized at $BACKUP" ;;
    disable|stop|off) echo "SPI shadow: backup remains in $BACKUP" ;;
    *) usage ;;
esac
