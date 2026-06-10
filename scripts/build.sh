#!/bin/bash
# KairosOS Build Script
# Builds a complete KairosOS ISO using Buildroot
#
# Prerequisites: Docker
# Usage: ./scripts/build.sh [--config <defconfig>] [--clean]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="${PROJECT_DIR}/output"
BUILDROOT_VERSION="2026.02"
BUILDROOT_DIR="${PROJECT_DIR}/buildroot-src"
DOCKER_IMAGE="kairosos-builder"
CONFIG="${1:-kairosos_defconfig}"

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RESET='\033[0m'

log_info()  { echo -e "${CYAN}⏺${RESET} $1"; }
log_ok()    { echo -e "${GREEN}✓${RESET} $1"; }
log_warn()  { echo -e "${YELLOW}⚠${RESET} $1"; }
log_error() { echo -e "${RED}✗${RESET} $1"; }

cleanup() {
    log_info "Cleaning up..."
    rm -rf "${OUTPUT_DIR}"
    log_ok "Clean."
}

build_docker() {
    log_info "Building Docker build image..."
    docker build -t "${DOCKER_IMAGE}" -f "${PROJECT_DIR}/Dockerfile" "${PROJECT_DIR}"
    log_ok "Docker image built."
}

fetch_buildroot() {
    if [ -d "${BUILDROOT_DIR}" ]; then
        log_info "Buildroot already fetched at ${BUILDROOT_DIR}"
        return
    fi

    log_info "Fetching Buildroot ${BUILDROOT_VERSION}..."
    git clone --depth 1 --branch "${BUILDROOT_VERSION}" \
        https://github.com/buildroot/buildroot.git "${BUILDROOT_DIR}" 2>/dev/null || {
        log_warn "Git clone failed, trying tarball..."
        curl -sL "https://buildroot.org/downloads/buildroot-${BUILDROOT_VERSION}.tar.gz" | tar xz
        mv "buildroot-${BUILDROOT_VERSION}" "${BUILDROOT_DIR}"
    }
    log_ok "Buildroot fetched."
}

build_iso() {
    log_info "Building KairosOS ISO..."
    log_info "Using config: ${CONFIG}"

    mkdir -p "${OUTPUT_DIR}"

    docker run --rm -it \
        -v "${PROJECT_DIR}:/build/kairosos" \
        -v "${OUTPUT_DIR}:/build/output" \
        "${DOCKER_IMAGE}" \
        /bin/bash -c "
            set -e
            cd /build

            # Symlink Buildroot if not exists
            if [ ! -d buildroot ]; then
                ln -s kairosos/buildroot-src buildroot 2>/dev/null || \
                (git clone --depth 1 --branch ${BUILDROOT_VERSION} \
                    https://github.com/buildroot/buildroot.git buildroot)
            fi

            cd buildroot
            make BR2_EXTERNAL=/build/kairosos/buildroot ${CONFIG}
            make BR2_EXTERNAL=/build/kairosos/buildroot
        "

    log_ok "Build complete!"
}

print_results() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${RESET}"
    echo -e "${GREEN}║       KairosOS Build Complete          ║${RESET}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${RESET}"
    echo ""

    if [ -f "${OUTPUT_DIR}/images/kairosos.iso" ]; then
        local SIZE=$(du -h "${OUTPUT_DIR}/images/kairosos.iso" | cut -f1)
        echo "  ISO:       ${OUTPUT_DIR}/images/kairosos.iso (${SIZE})"
    fi
    if [ -f "${OUTPUT_DIR}/images/rootfs.ext4" ]; then
        local SIZE=$(du -h "${OUTPUT_DIR}/images/rootfs.ext4" | cut -f1)
        echo "  RootFS:    ${OUTPUT_DIR}/images/rootfs.ext4 (${SIZE})"
    fi
    if [ -f "${OUTPUT_DIR}/images/bzImage" ]; then
        echo "  Kernel:    ${OUTPUT_DIR}/images/bzImage"
    fi
    echo ""
    echo "  Run with QEMU:"
    echo "    qemu-system-x86_64 -m 2048 -cdrom ${OUTPUT_DIR}/images/kairosos.iso -boot d"
    echo ""
}

# --- Main ---
main() {
    echo ""
    echo -e "${CYAN}  ╔══════════════════════════════════════╗${RESET}"
    echo -e "${CYAN}  ║         KairosOS Build System        ║${RESET}"
    echo -e "${CYAN}  ╚══════════════════════════════════════╝${RESET}"
    echo ""

    case "${1:-}" in
        --clean)
            cleanup
            exit 0
            ;;
        --help|-h)
            echo "Usage: $0 [--config <defconfig>] [--clean]"
            exit 0
            ;;
    esac

    build_docker
    fetch_buildroot
    build_iso
    print_results
}

main "$@"
