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
# kairos-selinux-compile - SELinux Policy Compilation
case "$ACTION" in
    status|stat) ls -la /etc/selinux/kairos/ 2>/dev/null || echo "no policy dir"; checkpolicy -V 2>/dev/null || echo "checkpolicy N/A" ;;
    enable|start|on) make -C /etc/selinux/kairos/ 2>/dev/null && log "policy compiled" || error "compile fail"; semodule -i /etc/selinux/kairos/kairos.pp 2>/dev/null && log "module loaded" || error "load fail" ;;
    disable|stop|off) semodule -r kairos 2>/dev/null && log "kairos module removed" || true ;;
    *) usage ;;
esac
