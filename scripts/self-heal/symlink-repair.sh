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
# kairos-symlink-repair - Broken Symlink Repair
case "$ACTION" in
    status|stat) find /etc/kairos -type l ! -exec test -e {} \; -print 2>/dev/null | head -20 || echo "no broken symlinks" ;;
    enable|start|on) for link in $(find /etc/kairos -type l ! -exec test -e {} \; 2>/dev/null); do target=$(readlink "$link"); log "broken: $link -> $target"; done ;;
    disable|stop|off) for link in $(find /etc/kairos -type l ! -exec test -e {} \; 2>/dev/null); do rm -f "$link" && log "removed broken: $link"; done ;;
    *) usage ;;
esac
