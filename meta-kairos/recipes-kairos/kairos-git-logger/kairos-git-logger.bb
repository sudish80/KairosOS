SUMMARY = "KairosOS Git-backed /etc tracker"
DESCRIPTION = "Daemon that monitors /etc for changes and creates \
semantic git commits for every system configuration change, enabling \
full audit trail and point-in-time recovery."
LICENSE = "Apache-2.0"
LIC_FILES_CHKSUM = "file://LICENSE;md5=3b83ef96387f14655fc854ddc3c6bd57"

inherit kairos-config cargo

SRC_URI = " \
    git://github.com/kairosos/kairos-git-logger.git;protocol=https;branch=main \
    file://kairos-git-logger.service \
"

SRCREV = "v0.2.0"

S = "${WORKDIR}/git"
CARGO_SRC_DIR = "src/daemons/kairos-git-logger"

DEPENDS += "libgit2 openssl zlib"

RDEPENDS:${PN} += "git"

do_install:append() {
    install -d ${D}${systemd_system_unitdir}
    install -m 0644 ${WORKDIR}/kairos-git-logger.service ${D}${systemd_system_unitdir}/

    # Initialize bare git store directory
    install -d ${D}${KAIROS_DATA_DIR}/git-store

    # AppArmor profile
    install -d ${D}/etc/apparmor.d
    install -m 0644 ${LAYERDIR}/../../config/apparmor/kairos-git-logger \
        ${D}/etc/apparmor.d/usr.sbin.kairos-git-logger
}

SYSTEMD_SERVICE:${PN} = "kairos-git-logger.service"
SYSTEMD_AUTO_ENABLE = "enable"

FILES:${PN} += " \
    ${KAIROS_DATA_DIR}/git-store \
    /etc/apparmor.d/usr.sbin.kairos-git-logger \
"
