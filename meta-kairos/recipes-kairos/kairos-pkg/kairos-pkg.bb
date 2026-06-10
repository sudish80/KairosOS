SUMMARY = "KairosOS Personal Knowledge Graph Service"
DESCRIPTION = "GraphRAG-powered semantic memory with SQLite+sqlite-vec. \
Provides hybrid retrieval (vector + FTS + graph neighborhood expansion) \
and deterministic entity extraction for personal knowledge management."
LICENSE = "Apache-2.0"
LIC_FILES_CHKSUM = "file://LICENSE;md5=3b83ef96387f14655fc854ddc3c6bd57"

inherit kairos-config

SRC_URI = " \
    git://github.com/kairosos/kairos-pkg.git;protocol=https;branch=main \
    file://kairos-pkg.service \
    file://kairos-pkg.socket \
"

SRCREV = "v0.2.0"

S = "${WORKDIR}/git"

inherit setuptools3

RDEPENDS:${PN} += " \
    python3-sqlite3 \
    python3-httpx \
    python3-pydantic \
    python3-pyyaml \
    python3-orjson \
    sqlite-vec \
"

do_install:append() {
    install -d ${D}${systemd_system_unitdir}
    install -m 0644 ${WORKDIR}/kairos-pkg.service ${D}${systemd_system_unitdir}/
    install -m 0644 ${WORKDIR}/kairos-pkg.socket ${D}${systemd_system_unitdir}/

    # Data directory
    install -d ${D}${KAIROS_DATA_DIR}/pkg

    # AppArmor profile (shared with kairos-hermes for now)
    install -d ${D}/etc/apparmor.d
}

SYSTEMD_SERVICE:${PN} = "kairos-pkg.socket kairos-pkg.service"
SYSTEMD_AUTO_ENABLE = "enable"

FILES:${PN} += " \
    ${KAIROS_DATA_DIR}/pkg \
"
