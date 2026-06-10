SUMMARY = "KairosOS eBPF Telemetry & Control Daemon"
DESCRIPTION = "Rust daemon providing eBPF-powered system observability \
with 6 BPF probes (execsnoop, tcptop, filemon, anomaly, schedlatency, oomkill) \
and an MCP interface for agent-driven telemetry and policy enforcement."
LICENSE = "Apache-2.0"
LIC_FILES_CHKSUM = "file://LICENSE;md5=3b83ef96387f14655fc854ddc3c6bd57"

inherit kairos-config cargo

SRC_URI = " \
    git://github.com/kairosos/kairos-bpf.git;protocol=https;branch=main \
    file://kairos-bpf.service \
"

SRCREV = "v0.2.0"

S = "${WORKDIR}/git"
CARGO_SRC_DIR = "src/daemons/kairos-bpf"

# Build dependencies
DEPENDS += " \
    clang-native \
    llvm-native \
    elfutils \
    zlib \
    openssl \
"

RDEPENDS:${PN} += " \
    bpftool \
    perl \
    bash \
"

do_install:append() {
    install -d ${D}${systemd_system_unitdir}
    install -m 0644 ${WORKDIR}/kairos-bpf.service ${D}${systemd_system_unitdir}/

    install -d ${D}${KAIROS_DATA_DIR}/bpf
    install -d ${D}${KAIROS_CONF_DIR}/bpf

    # Install AppArmor profile
    install -d ${D}/etc/apparmor.d
    install -m 0644 ${LAYERDIR}/../../config/apparmor/kairos-bpf \
        ${D}/etc/apparmor.d/usr.sbin.kairos-bpf
}

SYSTEMD_SERVICE:${PN} = "kairos-bpf.service"
SYSTEMD_AUTO_ENABLE = "enable"

FILES:${PN} += " \
    ${KAIROS_DATA_DIR}/bpf \
    ${KAIROS_CONF_DIR}/bpf \
    /etc/apparmor.d/usr.sbin.kairos-bpf \
"
