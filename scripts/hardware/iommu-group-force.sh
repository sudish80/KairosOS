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
# kairos-iommu-group-force - IOMMU Group Force
case "$ACTION" in
    status|stat) for g in /sys/kernel/iommu_groups/*; do echo "group $(basename $g): $(ls $g/devices 2>/dev/null | xargs)"; done ;;
    enable|start|on) echo "IOMMU groups: kernel-managed" ;;
    disable|stop|off) echo "IOMMU: kernel-managed, no disable" ;;
    *) usage ;;
esac
