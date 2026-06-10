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
# kairos-tpm-hierarchy-lock - TPM Hierarchy Lock
case "$ACTION" in
    status|stat) ls -la /sys/class/tpm/tpm*/ 2>/dev/null || echo "no TPM"; tpm2_getcap handles-persistent 2>/dev/null | head -5 || echo "tpm2-tools N/A" ;;
    enable|start|on) tpm2_hierarchycontrol -e -o lock -l 0x81000001 2>/dev/null && log "endorsement locked" || error "TPM lock fail"; tpm2_hierarchycontrol -o lock 2>/dev/null && log "owner locked" || true ;;
    disable|stop|off) echo "TPM unlocking requires platform auth" ;;
    *) usage ;;
esac
