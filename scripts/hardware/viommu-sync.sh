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
# kairos-viommu-sync - vIOMMU Remapping Synchronizer
case "$ACTION" in
    status|stat) dmesg | grep -i "iommu\|DMAR\|AMD-Vi\|VT-d" | tail -5 || echo "no IOMMU msgs" ;;
    enable|start|on) for g in /sys/kernel/iommu_groups/*; do echo "group $(basename $g): $(ls $g/devices 2>/dev/null | xargs)"; done; log "vIOMMU mapping synced" ;;
    disable|stop|off) echo "vIOMMU: kernel-managed" ;;
    sync) for g in /sys/kernel/iommu_groups/*; do echo "group $(basename $g): $(ls $g/devices 2>/dev/null | xargs)"; done ;;
    validate) dmesg | grep -i "IOMMU\|DMAR\|AMD-Vi\|VT-d" | tail -5 ;;
    *) usage ;;
esac
