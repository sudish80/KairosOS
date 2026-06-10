.PHONY: all build test clean distclean help rust-build rust-test python-test
.PHONY: kernel-build deb rpm arch docker-build docker-shell qemu qemu-iso
.PHONY: sbom security-scan docs format lint

all: rust-build kernel-build

# -----------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------
BUILDROOT_VERSION ?= 2026.02
BUILDROOT_DIR ?= buildroot-src
OUTPUT_DIR ?= output
DOCKER_IMAGE ?= kairosos-builder
CONFIG ?= kairosos_defconfig
RUST_TARGET ?= x86_64-unknown-linux-gnu

# -----------------------------------------------------------------------
# Help
# -----------------------------------------------------------------------
help:
	@echo "KairosOS Build System"
	@echo ""
	@echo "BUILD TARGETS:"
	@echo "  make rust-build        - Build all Rust daemons and MCP servers (cargo build --release)"
	@echo "  make python-build      - Build Python AI service wheels"
	@echo "  make kernel-build      - Build all kernel modules"
	@echo "  make build             - Full build: rust + kernel + python"
	@echo ""
	@echo "TEST TARGETS:"
	@echo "  make rust-test         - Run all Rust tests (cargo test --workspace)"
	@echo "  make python-test       - Run all Python tests (pytest)"
	@echo "  make test              - Run all tests"
	@echo ""
	@echo "QUALITY TARGETS:"
	@echo "  make format            - Format Rust code (cargo fmt)"
	@echo "  make lint              - Run all linters (clippy, ruff, shellcheck)"
	@echo "  make docs              - Build Rust API documentation"
	@echo "  make sbom              - Generate SPDX SBOM"
	@echo "  make security-scan     - Run cargo-audit vulnerability scan"
	@echo ""
	@echo "PACKAGING TARGETS:"
	@echo "  make deb               - Build Debian packages"
	@echo "  make rpm               - Build RPM packages"
	@echo "  make arch              - Build Arch Linux package"
	@echo ""
	@echo "IMAGE TARGETS:"
	@echo "  make docker-build      - Build the Docker builder image"
	@echo "  make docker-shell      - Start a shell in the builder container"
	@echo "  make build             - Build full KairosOS ISO via Buildroot (inside Docker)"
	@echo "  make qemu              - Boot the built ISO in QEMU"
	@echo ""
	@echo "SYSTEM TARGETS:"
	@echo "  make clean             - Clean build artifacts"
	@echo "  make distclean         - Full clean including Buildroot source"
	@echo "  make install           - Install daemon binaries to /usr/bin"
	@echo ""
	@echo "VARIABLES:"
	@echo "  CONFIG=name        - Buildroot defconfig name (default: kairosos_defconfig)"
	@echo "  RUST_TARGET=triple - Cross-compilation target"
	@echo "  KDIR=/path         - Kernel headers path for module builds"

# -----------------------------------------------------------------------
# Build: Rust daemons + MCP servers
# -----------------------------------------------------------------------
rust-build:
	@echo "=== Building Rust workspace ==="
	cd src && cargo build --release --workspace --target $(RUST_TARGET)
	@echo "=== Building MCP servers ==="
	cd src/mcp-servers && \
	for d in */; do \
		if [ -f "$${d}Cargo.toml" ]; then \
			cargo build --release --manifest-path "$${d}Cargo.toml" --target $(RUST_TARGET); \
		fi; \
	done

# -----------------------------------------------------------------------
# Build: Python AI wheels
# -----------------------------------------------------------------------
python-build:
	@echo "=== Building Python AI wheels ==="
	mkdir -p build/python
	for d in ai/*/; do \
		if [ -f "$${d}pyproject.toml" ]; then \
			cd "$$d" && python3 -m build --wheel --outdir ../../build/python/ && cd ../..; \
		fi; \
	done

# -----------------------------------------------------------------------
# Build: Kernel modules
# -----------------------------------------------------------------------
kernel-build:
	@echo "=== Building kernel modules ==="
	for d in kernel/*/; do \
		echo "  Building $${d}..."; \
		$(MAKE) -C "$$d"; \
	done

# -----------------------------------------------------------------------
# Build: Full
# -----------------------------------------------------------------------
build: rust-build kernel-build python-build
	@echo "=== Full build complete ==="

# -----------------------------------------------------------------------
# Test: Rust
# -----------------------------------------------------------------------
rust-test:
	@echo "=== Running Rust tests ==="
	cd src && cargo test --workspace --target $(RUST_TARGET)

# -----------------------------------------------------------------------
# Test: Python
# -----------------------------------------------------------------------
python-test:
	@echo "=== Running Python tests ==="
	python3 -m pytest tests/ ai/*/tests/ -v --timeout=30

# -----------------------------------------------------------------------
# Test: All
# -----------------------------------------------------------------------
test: rust-test python-test

# -----------------------------------------------------------------------
# Quality: Format
# -----------------------------------------------------------------------
format:
	cd src && cargo fmt --all

# -----------------------------------------------------------------------
# Quality: Lint
# -----------------------------------------------------------------------
lint:
	@echo "=== Rust clippy ==="
	cd src && cargo clippy --workspace --all-targets -- -D warnings
	@echo "=== Python ruff ==="
	ruff check ai/ tests/
	@echo "=== ShellCheck ==="
	shellcheck scripts/**/*.sh

# -----------------------------------------------------------------------
# Quality: Documentation
# -----------------------------------------------------------------------
docs:
	cd src && cargo doc --workspace --no-deps
	@echo "Docs available at src/target/doc/index.html"

# -----------------------------------------------------------------------
# Quality: SBOM
# -----------------------------------------------------------------------
sbom:
	cargo install cargo-cyclonedx 2>/dev/null || true
	cd src && cargo cyclonedx --workspace --output-format json

# -----------------------------------------------------------------------
# Quality: Security scan
# -----------------------------------------------------------------------
security-scan:
	cargo install cargo-audit 2>/dev/null || true
	cd src && cargo audit

# -----------------------------------------------------------------------
# Packaging: Debian
# -----------------------------------------------------------------------
deb:
	cd packaging/debian && dpkg-buildpackage -us -uc -b

# -----------------------------------------------------------------------
# Packaging: RPM
# -----------------------------------------------------------------------
rpm:
	mkdir -p build/rpm
	rpmbuild -bb packaging/rpm/kairosos.spec --define "_topdir $(PWD)/build/rpm"

# -----------------------------------------------------------------------
# Packaging: Arch
# -----------------------------------------------------------------------
arch:
	cd packaging/arch && makepkg -si

# -----------------------------------------------------------------------
# Docker
# -----------------------------------------------------------------------
buildroot-src:
	@echo "Fetching Buildroot $(BUILDROOT_VERSION)..."
	git clone --depth 1 --branch $(BUILDROOT_VERSION) \
		https://github.com/buildroot/buildroot.git $(BUILDROOT_DIR)

docker-build:
	docker build -t $(DOCKER_IMAGE) -f Dockerfile .

docker-shell: docker-build
	docker run --rm -it \
		-v "$(PWD):/build/kairosos" \
		-v "$(PWD)/$(OUTPUT_DIR):/build/output" \
		$(DOCKER_IMAGE) \
		/bin/bash

build-iso: docker-build
	@echo "Building KairosOS ISO..."
	docker run --rm -it \
		-v "$(PWD):/build/kairosos" \
		-v "$(PWD)/$(OUTPUT_DIR):/build/output" \
		$(DOCKER_IMAGE) \
		/bin/bash -c "\
			cd /build && \
			if [ ! -d buildroot-$(BUILDROOT_VERSION) ]; then \
				git clone --depth 1 --branch $(BUILDROOT_VERSION) \
					https://github.com/buildroot/buildroot.git buildroot-$(BUILDROOT_VERSION) || \
				(curl -sL https://buildroot.org/downloads/buildroot-$(BUILDROOT_VERSION).tar.gz | tar xz); \
			fi && \
			cd buildroot-$(BUILDROOT_VERSION) && \
			make BR2_EXTERNAL=/build/kairosos/buildroot $(CONFIG) && \
			make BR2_EXTERNAL=/build/kairosos/buildroot"

# -----------------------------------------------------------------------
# QEMU
# -----------------------------------------------------------------------
qemu:
	qemu-system-x86_64 \
		-m 2048 -smp 2 \
		-drive file=$(OUTPUT_DIR)/images/rootfs.ext4,format=raw,if=virtio \
		-kernel $(OUTPUT_DIR)/images/bzImage \
		-append "root=/dev/vda console=ttyS0" \
		-nographic

qemu-iso:
	qemu-system-x86_64 \
		-m 2048 -smp 2 \
		-cdrom $(OUTPUT_DIR)/images/kairosos.iso -boot d

# -----------------------------------------------------------------------
# Install
# -----------------------------------------------------------------------
install:
	install -d $(DESTDIR)/usr/bin
	for f in src/target/release/kairos-*; do \
		[ -x "$$f" ] && install -m 0755 "$$f" $(DESTDIR)/usr/bin/; \
	done
	install -d $(DESTDIR)/usr/sbin
	for d in src/mcp-servers/*/; do \
		name=$$(basename "$$d"); binary="$${name%-server}"; \
		bin="src/mcp-servers/$$name/target/release/kairos-$${binary}-mcp"; \
		[ -x "$$bin" ] && install -m 0755 "$$bin" $(DESTDIR)/usr/sbin/; \
	done

# -----------------------------------------------------------------------
# Clean
# -----------------------------------------------------------------------
clean:
	rm -rf $(OUTPUT_DIR) build
	cd src && cargo clean 2>/dev/null || true
	for d in kernel/*/; do $(MAKE) -C "$$d" clean 2>/dev/null || true; done

distclean: clean
	rm -rf $(BUILDROOT_DIR) buildroot-*

.PHONY: all build test clean distclean help rust-build rust-test python-test
.PHONY: kernel-build deb rpm arch docker-build docker-shell qemu qemu-iso
.PHONY: sbom security-scan docs format lint install
