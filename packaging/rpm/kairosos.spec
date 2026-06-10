# =============================================================================
# KairosOS RPM Spec — Fedora / EL / OpenSUSE
# =============================================================================
%define _prefix /usr
%define _sysconfdir /etc
%define _unitdir %{_prefix}/lib/systemd/system
%define _apparmordir /etc/apparmor.d

Name: kairosos
Version: 1.0.0
Release: 1%{?dist}
Summary: KairosOS autonomous operating system suite
License: GPL-2.0-only
URL: https://kairosos.org
Source0: %{name}-%{version}.tar.gz
BuildRequires: cargo, rustc, python3, python3-pip, python3-build
BuildRequires: gcc, make, kernel-devel, perl, bash
Requires:       systemd, kmod, util-linux
Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd

%description
KairosOS is a production-hardened autonomous operating system with
20 Rust daemons, 8 Python AI microservices, 7 C kernel modules,
74 shell scripts for security/network/hardware management,
35 systemd service units, and 22 AppArmor profiles.

%package daemons
Summary: KairosOS core Rust daemons
Requires: %{_prefix}/sbin/kairos-*-mcp

%description daemons
Production-hardened system daemons for the KairosOS autonomous OS.
Includes BPF telemetry, declarative config, recovery, inference hub,
mesh networking, orchestrator, TUI, and more.

%package ai-services
Summary: KairosOS AI microservices
BuildArch: noarch
Requires: python3

%description ai-services
Python-based AI microservices for confidence scoring, context management,
supervision, knowledge graphs, and telemetry aggregation.

%package kernel-modules
Summary: KairosOS kernel modules
Requires: kmod

%description kernel-modules
Custom kernel modules for dm-verity integrity, TPM PCR binding,
ECC memory error handling, PTP hardware sync, framebuffer,
IOMMU DMA isolation, and PROCHOT thermal management.

%package scripts
Summary: KairosOS management shell scripts
BuildArch: noarch
Requires: bash, systemd, kmod, util-linux

%description scripts
74 production-hardened shell scripts for security hardening,
network configuration, hardware tuning, storage management,
and self-healing system recovery.

%prep
%setup -q

%build
# Rust daemons
cd src
cargo build --release --workspace
# MCP servers
cd mcp-servers
for d in */; do
    if [ -f "${d}Cargo.toml" ]; then
        cargo build --release --manifest-path "${d}Cargo.toml"
    fi
done
cd ../..
# Python wheels
for d in ai/*/; do
    if [ -f "${d}pyproject.toml" ]; then
        cd "$d"
        python3 -m build --wheel --outdir ../../build/python/
        cd ../..
    fi
done
# Kernel modules
for d in kernel/*/; do
    make -C "$d"
done

%install
# Clear root
rm -rf %{buildroot}
# Daemon binaries
install -d %{buildroot}%{_bindir}
for f in src/target/release/kairos-*; do
    [ -x "$f" ] && install -m 0755 "$f" %{buildroot}%{_bindir}/
done
# MCP server binaries
install -d %{buildroot}%{_sbindir}
for d in src/mcp-servers/*/; do
    name=$(basename "$d")
    binary="${name%-server}"
    binfile="src/mcp-servers/${name}/target/release/kairos-${binary}-mcp"
    [ -x "$binfile" ] && install -m 0755 "$binfile" %{buildroot}%{_sbindir}/
done
# AI services
install -d %{buildroot}/opt/kairos/ai
cp -r ai/* %{buildroot}/opt/kairos/ai/
# Systemd units
install -d %{buildroot}%{_unitdir}
cp config/systemd/*.service %{buildroot}%{_unitdir}/
# AppArmor profiles
install -d %{buildroot}%{_apparmordir}
cp config/apparmor/* %{buildroot}%{_apparmordir}/
# Scripts
install -d %{buildroot}%{_prefix}/lib/kairos/scripts
cp -r scripts/* %{buildroot}%{_prefix}/lib/kairos/scripts/
# Kernel modules
install -d %{buildroot}/lib/modules/%{_kernel_version}/kairos
for f in kernel/*/*.ko; do
    install -m 0644 "$f" %{buildroot}/lib/modules/%{_kernel_version}/kairos/
done
# Config
install -d %{buildroot}%{_sysconfdir}/kairos
cp -r config/* %{buildroot}%{_sysconfdir}/kairos/

%post
%systemd_post kairos-bpf.service kairos-mcp.service kairos-apply.service
%systemd_post kairos-git-logger.service kairos-inference-hub.service
%systemd_post kairos-recovery.service kairos-tui.service
%systemd_post kairos-orchestrator.service kairos-mesh.service kairos-db.service
%systemd_post kairos-fb.service kairos-llm.service kairos-build.service

%preun
%systemd_preun kairos-bpf.service kairos-mcp.service kairos-apply.service
%systemd_preun kairos-git-logger.service kairos-inference-hub.service
%systemd_preun kairos-recovery.service kairos-tui.service
%systemd_preun kairos-orchestrator.service kairos-mesh.service kairos-db.service

%postun
%systemd_postun_with_restart kairos-bpf.service kairos-mcp.service kairos-apply.service
%systemd_postun_with_restart kairos-git-logger.service kairos-inference-hub.service
%systemd_postun_with_restart kairos-recovery.service kairos-tui.service

%files
%license LICENSE
%doc README.md CHECKLIST.md
%{_bindir}/kairos-*
%{_sbindir}/kairos-*-mcp
%{_unitdir}/*.service
%{_apparmordir}/*
%{_sysconfdir}/kairos/*
%{_prefix}/lib/kairos/scripts/*
/lib/modules/*/kairos/*.ko
/opt/kairos/ai/*

%files daemons
%{_bindir}/kairos-*

%files ai-services
/opt/kairos/ai/*

%files kernel-modules
/lib/modules/*/kairos/*.ko

%files scripts
%{_prefix}/lib/kairos/scripts/*

%changelog
* Wed Jun 10 2026 KairosOS Team <dev@kairosos.org> - 1.0.0-1
- Initial release
- 20 Rust daemons, 8 Python AI services, 7 kernel modules, 74 scripts
