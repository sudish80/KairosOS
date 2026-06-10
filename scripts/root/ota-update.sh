#!/bin/bash
# KairosOS OTA Update Orchestrator
# Usage: ota-update.sh [check|download|apply|rollback|status|set-channel]
set -euo pipefail
VERSION="1.0.0"
SCRIPT="${0##*/}"

log()  { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }
error() { log "ERROR: $*" >&2; }

usage() {
    cat <<EOF
KairosOS OTA Update Manager v${VERSION}
Usage: ${SCRIPT} [command]

Commands:
  check           Check for available updates
  download        Download available update
  apply           Apply downloaded update to inactive slot
  rollback        Rollback to previous slot
  status          Show update status
  set-channel     Set update channel (stable|beta|nightly)
  set-auto-apply  Enable/disable automatic apply (on|off)
  help            Show this help

EOF
    exit 0
}

KAIROS_BIN="${KAIROS_BIN:-/usr/bin}"
RECOVERY_BIN="${KAIROS_BIN}/kairos-recovery"
UPDATES_DIR="/var/lib/kairos/updates"
CONFIG_FILE="/etc/kairos/recovery.toml"

check_deps() {
    if [ ! -x "${RECOVERY_BIN}" ]; then
        error "kairos-recovery not found at ${RECOVERY_BIN}"
        exit 1
    fi
}

cmd_check() {
    log "Checking for updates..."
    ${RECOVERY_BIN} --check-update
}

cmd_download() {
    log "Downloading available update..."
    ${RECOVERY_BIN} --download-update
}

cmd_apply() {
    log "Applying update to inactive slot..."
    if [ ! -d "${UPDATES_DIR}" ] || [ -z "$(ls -A "${UPDATES_DIR}" 2>/dev/null)" ]; then
        error "No downloaded update found in ${UPDATES_DIR}"
        log "Run '${SCRIPT} download' first"
        exit 1
    fi
    ${RECOVERY_BIN} --apply-update
    log "Update applied. Reboot to activate: sudo reboot"
}

cmd_rollback() {
    log "Rolling back to previous slot..."
    ${RECOVERY_BIN} --rollback
    log "Rollback complete. Reboot to activate: sudo reboot"
}

cmd_status() {
    log "Update status:"
    ${RECOVERY_BIN} --update-status 2>/dev/null || echo "  update-manager: idle"

    local active_slot=""
    if [ -f /usr/lib/kairos/slot_a ]; then
        active_slot="A"
    else
        active_slot="B"
    fi
    echo "  active-slot: ${active_slot}"

    if [ -f "${UPDATES_DIR}/manifest.json" ]; then
        echo "  cached-manifest: $(cat "${UPDATES_DIR}/manifest.json" | python3 -c "import sys,json; m=json.load(sys.stdin); print(f'{m.get(\"version\",\"?\")} ({m.get(\"channel\",\"?\")})')" 2>/dev/null || echo "present")"
    else
        echo "  cached-manifest: none"
    fi

    if ${RECOVERY_BIN} --check-update 2>/dev/null; then
        :
    fi
}

cmd_set_channel() {
    local channel="${1:-}"
    if [ -z "${channel}" ] || [[ ! "${channel}" =~ ^(stable|beta|nightly)$ ]]; then
        error "Channel must be one of: stable, beta, nightly"
        exit 1
    fi
    if [ -f "${CONFIG_FILE}" ]; then
        sed -i "s/channel = \".*\"/channel = \"${channel}\"/" "${CONFIG_FILE}"
    fi
    log "Update channel set to: ${channel}"
}

cmd_set_auto_apply() {
    local mode="${1:-}"
    case "${mode}" in
        on|true|1) val="true" ;;
        off|false|0) val="false" ;;
        *) error "Usage: ${SCRIPT} set-auto-apply [on|off]"; exit 1 ;;
    esac
    if [ -f "${CONFIG_FILE}" ]; then
        sed -i "s/auto_apply = .*/auto_apply = ${val}/" "${CONFIG_FILE}"
    fi
    log "Auto-apply set to: ${val}"
}

main() {
    check_deps
    local cmd="${1:-help}"
    shift 2>/dev/null || true
    case "${cmd}" in
        check) cmd_check ;;
        download) cmd_download ;;
        apply) cmd_apply ;;
        rollback) cmd_rollback ;;
        status) cmd_status ;;
        set-channel) cmd_set_channel "$@" ;;
        set-auto-apply) cmd_set_auto_apply "$@" ;;
        help|--help|-h) usage ;;
        *) error "Unknown command: ${cmd}"; usage ;;
    esac
}

main "$@"
