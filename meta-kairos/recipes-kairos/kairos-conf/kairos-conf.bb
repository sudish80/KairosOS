SUMMARY = "KairosOS Declarative Config Engine"
DESCRIPTION = "Atomic configuration management daemon that applies, validates, \
and rolls back system state from declarative configuration files. \
Supports YAML/TOML/JSON formats with SHA256 content-addressed generations."
LICENSE = "Apache-2.0"
LIC_FILES_CHKSUM = "file://LICENSE;md5=3b83ef96387f14655fc854ddc3c6bd57"

inherit kairos-config cargo

SRC_URI = " \
    git://github.com/kairosos/kairos-conf.git;protocol=https;branch=main \
    file://kairos-apply.service \
    file://kairos-apply.timer \
"

SRCREV = "v0.2.0"

S = "${WORKDIR}/git"
CARGO_SRC_DIR = "src/daemons/kairos-apply"

RDEPENDS:${PN} += " \
    kairos-git-logger \
    python3-core \
    python3-pyyaml \
"

do_install:append() {
    install -d ${D}${systemd_system_unitdir}
    install -m 0644 ${WORKDIR}/kairos-apply.service ${D}${systemd_system_unitdir}/
    install -m 0644 ${WORKDIR}/kairos-apply.timer ${D}${systemd_system_unitdir}/

    # Default declarative configuration
    install -d ${D}${KAIROS_CONF_DIR}
    install -m 0644 ${LAYERDIR}/../../config/declarative/default-configuration.nix \
        ${D}${KAIROS_CONF_DIR}/configuration.nix

    # AppArmor profile
    install -d ${D}/etc/apparmor.d
    install -m 0644 ${LAYERDIR}/../../config/apparmor/kairos-apply \
        ${D}/etc/apparmor.d/usr.sbin.kairos-apply
}

SYSTEMD_SERVICE:${PN} = "kairos-apply.service kairos-apply.timer"
SYSTEMD_AUTO_ENABLE = "enable"

FILES:${PN} += " \
    ${KAIROS_CONF_DIR}/configuration.nix \
    /etc/apparmor.d/usr.sbin.kairos-apply \
"
