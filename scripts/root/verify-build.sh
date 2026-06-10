#!/bin/bash
# =============================================================================
# KairosOS Build Verification Script
# Runs full build + test pipeline on native Linux.
# Usage: sudo ./verify-build.sh [--skip-kernel] [--skip-python] [--quick]
# =============================================================================
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; YELLOW='\033[1;33m'; RESET='\033[0m'
PASS=0; FAIL=0; WARN=0

pass() { PASS=$((PASS+1)); echo -e "${GREEN}[PASS]${RESET} $1"; }
fail() { FAIL=$((FAIL+1)); echo -e "${RED}[FAIL]${RESET} $1"; }
warn() { WARN=$((WARN+1)); echo -e "${YELLOW}[WARN]${RESET} $1"; }
info() { echo -e "${CYAN}[INFO]${RESET} $1"; }

SKIP_KERNEL=false; SKIP_PYTHON=false; QUICK=false
for arg in "$@"; do
    case "$arg" in
        --skip-kernel) SKIP_KERNEL=true ;;
        --skip-python) SKIP_PYTHON=true ;;
        --quick) QUICK=true ;;
    esac
done

# ---------------------------------------------------------------------------
# Prerequisites check
# ---------------------------------------------------------------------------
info "=== Step 0: Prerequisites ==="
for cmd in cargo rustc python3 make docker; do
    if command -v "$cmd" &>/dev/null; then
        pass "$cmd found: $($cmd --version 2>/dev/null | head -1)"
    else
        fail "$cmd not found"
    fi
done

# ---------------------------------------------------------------------------
# Step 1: Rust build
# ---------------------------------------------------------------------------
info "=== Step 1: Rust Workspace Build ==="
cd src

if cargo build --release --workspace 2>&1; then
    pass "Rust workspace build (release)"
else
    fail "Rust workspace build"
fi

if cargo clippy --workspace --all-targets -- -D warnings 2>&1; then
    pass "Rust clippy (no warnings)"
else
    fail "Rust clippy"
fi

if cargo fmt --all --check 2>&1; then
    pass "Rust fmt"
else
    fail "Rust fmt (run 'cargo fmt' to fix)"
fi

# ---------------------------------------------------------------------------
# Step 2: Rust tests
# ---------------------------------------------------------------------------
info "=== Step 2: Rust Tests ==="
if cargo test --workspace 2>&1; then
    pass "Rust tests"
else
    fail "Rust tests"
fi

# ---------------------------------------------------------------------------
# Step 3: MCP servers build
# ---------------------------------------------------------------------------
info "=== Step 3: MCP Servers Build ==="
cd mcp-servers
for d in */; do
    if [ -f "${d}Cargo.toml" ]; then
        if cargo build --release --manifest-path "${d}Cargo.toml" 2>&1; then
            pass "MCP server: ${d%/}"
        else
            fail "MCP server: ${d%/}"
        fi
    fi
done
cd ..

# ---------------------------------------------------------------------------
# Step 4: Python tests
# ---------------------------------------------------------------------------
if [ "$SKIP_PYTHON" = false ]; then
    info "=== Step 4: Python Tests ==="
    cd ..
    pip install ruff pytest pytest-asyncio pytest-timeout 2>/dev/null || true

    if ruff check ai/ 2>&1; then
        pass "Python ruff lint"
    else
        warn "Python ruff lint (issues found)"
    fi

    if python3 -m pytest tests/ ai/*/tests/ -v --timeout=30 2>&1; then
        pass "Python tests"
    else
        fail "Python tests"
    fi
    cd src
fi

cd ..

# ---------------------------------------------------------------------------
# Step 5: Shell check
# ---------------------------------------------------------------------------
if command -v shellcheck &>/dev/null; then
    info "=== Step 5: ShellCheck ==="
    if shellcheck scripts/**/*.sh 2>&1; then
        pass "ShellCheck"
    else
        warn "ShellCheck (issues found)"
    fi
else
    warn "shellcheck not installed, skipping"
fi

# ---------------------------------------------------------------------------
# Step 6: Kernel modules build
# ---------------------------------------------------------------------------
if [ "$SKIP_KERNEL" = false ]; then
    info "=== Step 6: Kernel Modules Build ==="
    if [ ! -d /lib/modules/"$(uname -r)"/build ]; then
        warn "Kernel headers not installed, skipping kernel build"
        warn "Install with: sudo apt install linux-headers-\$(uname -r)"
    else
        for d in kernel/*/; do
            if make -C "$d" 2>&1; then
                pass "Kernel module: ${d%/}"
            else
                fail "Kernel module: ${d%/}"
            fi
        done
    fi
fi

# ---------------------------------------------------------------------------
# Step 7: Rust security audit
# ---------------------------------------------------------------------------
info "=== Step 7: Security Audit ==="
cd src
if cargo audit 2>&1; then
    pass "cargo-audit (no vulnerabilities)"
else
    warn "cargo-audit found vulnerabilities (review advisories)"
fi

# ---------------------------------------------------------------------------
# Step 8: Documentation build
# ---------------------------------------------------------------------------
if [ "$QUICK" = false ]; then
    info "=== Step 8: Documentation ==="
    if cargo doc --workspace --no-deps 2>&1; then
        pass "Rust documentation"
    else
        warn "Rust documentation (warnings)"
    fi
fi

cd ..

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "========================================"
echo "  KairosOS Build Verification Results"
echo "========================================"
echo "  PASS: ${PASS}"
echo "  FAIL: ${FAIL}"
echo "  WARN: ${WARN}"
echo "========================================"

if [ "$FAIL" -gt 0 ]; then
    echo -e "${RED}BUILD VERIFICATION FAILED${RESET}"
    exit 1
else
    echo -e "${GREEN}BUILD VERIFICATION PASSED${RESET}"
    exit 0
fi
