#!/bin/bash
# KairosOS post-image script
# Creates the final ISO and additional image formats

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BOARD_DIR="${SCRIPT_DIR}"
IMAGES_DIR="${BINARIES_DIR:-$1}"

if [ -z "$IMAGES_DIR" ]; then
    echo "Usage: $0 <images-dir>"
    exit 1
fi

echo "KairosOS: Post-image processing..."

# Copy additional files to the image
mkdir -p "${IMAGES_DIR}/kairosos"

# Generate checksums
cd "${IMAGES_DIR}"
sha256sum kairosos.iso > kairosos.iso.sha256 2>/dev/null || true
sha256sum rootfs.ext4 > rootfs.ext4.sha256 2>/dev/null || true

echo "KairosOS: Image built successfully!"
echo "  ISO: ${IMAGES_DIR}/kairosos.iso"
echo "  RootFS: ${IMAGES_DIR}/rootfs.ext4"
echo ""
echo "  Boot with QEMU:"
echo "    qemu-system-x86_64 -m 2048 -cdrom ${IMAGES_DIR}/kairosos.iso -boot d"
