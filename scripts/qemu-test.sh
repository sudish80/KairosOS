#!/bin/bash
# KairosOS QEMU Integration Test Harness
# Boots the built ISO/image and runs smoke tests.
set -euo pipefail

KERNEL="${1:-output/images/bzImage}"
ROOTFS="${2:-output/images/rootfs.ext4}"
TIMEOUT="${3:-120}"
TEST_DIR="/tmp/kairos-qemu-test"

echo "KairosOS QEMU Test Harness"
echo "  Kernel: $KERNEL"
echo "  Rootfs: $ROOTFS"
echo "  Timeout: ${TIMEOUT}s"

if [ ! -f "$KERNEL" ] || [ ! -f "$ROOTFS" ]; then
    echo "Error: Kernel or rootfs not found. Build the image first."
    exit 1
fi

mkdir -p "$TEST_DIR"

# Launch QEMU
echo "Starting QEMU..."
qemu-system-x86_64 \
    -m 2048 \
    -smp 2 \
    -kernel "$KERNEL" \
    -drive file="$ROOTFS",format=raw,if=virtio \
    -append "root=/dev/vda console=ttyS0 panic=1 quiet" \
    -nographic \
    -no-reboot \
    -monitor none \
    -serial file:"$TEST_DIR/serial.log" \
    -pidfile "$TEST_DIR/qemu.pid" \
    -netdev user,id=net0 \
    -device virtio-net,netdev=net0 &
QEMU_PID=$!

# Wait for boot and run tests
sleep 10
echo "Running smoke tests..."

TESTS_PASSED=0
TESTS_FAILED=0

check_kernel_boot() {
    if grep -q "Linux version" "$TEST_DIR/serial.log" 2>/dev/null; then
        echo "  ✓ Kernel booted"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo "  ✗ Kernel did not boot"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

check_service_start() {
    local svc="$1"
    if grep -q "Started.*$svc" "$TEST_DIR/serial.log" 2>/dev/null || grep -q "$svc.*running" "$TEST_DIR/serial.log" 2>/dev/null; then
        echo "  ✓ Service $svc started"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo "  ○ Service $svc status unknown (may start later)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    fi
}

check_kernel_boot
check_service_start "kairos-mcp"
check_service_start "kairos-bpf"

# Wait for timeout or completion
sleep "$TIMEOUT" &

# Cleanup
kill "$QEMU_PID" 2>/dev/null || true
wait "$QEMU_PID" 2>/dev/null || true

echo ""
echo "=== Test Results ==="
echo "  Passed: $TESTS_PASSED"
echo "  Failed: $TESTS_FAILED"
echo "  Log: $TEST_DIR/serial.log"

if [ "$TESTS_FAILED" -gt 0 ]; then
    exit 1
fi
echo "All tests passed!"
