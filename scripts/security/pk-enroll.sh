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
# kairos-pk-enroll - UEFI Secure Boot PK Enrollment
case "$ACTION" in
    status|stat) mokutil --sb-state 2>/dev/null || echo "mokutil N/A"; ls -la /etc/kairos/secureboot/ 2>/dev/null || echo "no keys" ;;
    enable|start|on) for key in PK KEK db; do [ -f "/etc/kairos/secureboot/${key}.auth" ] || continue; efi-updatevar -e -k "/etc/kairos/secureboot/${key}.key" -c "/etc/kairos/secureboot/${key}.crt" "$key" 2>/dev/null && log "$key enrolled" || error "enroll $key"; done ;;
    disable|stop|off) echo "UEFI SB: cannot unenroll (hardware)" ;;
    *) usage ;;
esac
