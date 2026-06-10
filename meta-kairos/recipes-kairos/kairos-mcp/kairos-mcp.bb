SUMMARY = "KairosOS MCP Protocol Router"
DESCRIPTION = "Universal agent↔system communication router implementing \
the Model Context Protocol. Routes capabilities between AI agents and \
all KairosOS subsystems via Unix sockets and HTTP transports."
LICENSE = "Apache-2.0"
LIC_FILES_CHKSUM = "file://LICENSE;md5=3b83ef96387f14655fc854ddc3c6bd57"

inherit kairos-config cargo

SRC_URI = " \
    git://github.com/kairosos/kairos-mcp.git;protocol=https;branch=main \
    file://kairos-mcp.service \
"

SRCREV = "v0.2.0"

S = "${WORKDIR}/git"
CARGO_SRC_DIR = "src/daemons/kairos-mcp"

DEPENDS += "openssl"

RDEPENDS:${PN} += " \
    kairos-bpf \
    kairos-conf \
    kairos-git-logger \
    kairos-pkg \
"

do_install:append() {
    install -d ${D}${systemd_system_unitdir}
    install -m 0644 ${WORKDIR}/kairos-mcp.service ${D}${systemd_system_unitdir}/

    # MCP socket directory created by tmpfiles
    install -d ${D}${systemd_system_unitdir}
    install -m 0644 ${LAYERDIR}/files/kairos-mcp-tmpfiles.conf \
        ${D}/usr/lib/tmpfiles.d/kairos-mcp.conf

    # AppArmor profile
    install -d ${D}/etc/apparmor.d
    install -m 0644 ${LAYERDIR}/../../config/apparmor/kairos-mcp \
        ${D}/etc/apparmor.d/usr.sbin.kairos-mcp
}

SYSTEMD_SERVICE:${PN} = "kairos-mcp.service"
SYSTEMD_AUTO_ENABLE = "enable"

FILES:${PN} += " \
    /usr/lib/tmpfiles.d/kairos-mcp.conf \
    /etc/apparmor.d/usr.sbin.kairos-mcp \
"
