SUMMARY = "KairosOS Hermes AI Agent"
DESCRIPTION = "Primary AI assistant for KairosOS. Provides natural language \
system management, knowledge graph integration, and automated operations \
via the MCP protocol across all KairosOS subsystems."
LICENSE = "Apache-2.0"
LIC_FILES_CHKSUM = "file://LICENSE;md5=3b83ef96387f14655fc854ddc3c6bd57"

inherit kairos-config

SRC_URI = " \
    git://github.com/NousResearch/hermes.git;protocol=https;branch=main \
    file://kairos-hermes.service \
    file://hermes-config.yaml \
"

SRCREV = "main"

S = "${WORKDIR}/git"

inherit setuptools3

RDEPENDS:${PN} += " \
    python3-httpx \
    python3-pydantic \
    python3-pyyaml \
    python3-aiohttp \
    ollama \
    kairos-mcp \
"

do_install:append() {
    install -d ${D}${systemd_system_unitdir}
    install -m 0644 ${WORKDIR}/kairos-hermes.service ${D}${systemd_system_unitdir}/

    # Agent configuration
    install -d ${D}${KAIROS_CONF_DIR}/agent
    install -m 0644 ${WORKDIR}/hermes-config.yaml ${D}${KAIROS_CONF_DIR}/agent/config.yaml

    # Install skills
    install -d ${D}${KAIROS_CONF_DIR}/agent/skills
    for skill in ${LAYERDIR}/../../agent/skills/*.md; do
        install -m 0644 "$skill" ${D}${KAIROS_CONF_DIR}/agent/skills/
    done

    # AppArmor profile
    install -d ${D}/etc/apparmor.d
    install -m 0644 ${LAYERDIR}/../../config/apparmor/kairos-hermes \
        ${D}/etc/apparmor.d/usr.bin.hermes

    # Ollama AppArmor profile
    install -m 0644 ${LAYERDIR}/../../config/apparmor/ollama \
        ${D}/etc/apparmor.d/usr.bin.ollama
}

SYSTEMD_SERVICE:${PN} = "kairos-hermes.service"
SYSTEMD_AUTO_ENABLE = "enable"

FILES:${PN} += " \
    ${KAIROS_CONF_DIR}/agent/config.yaml \
    ${KAIROS_CONF_DIR}/agent/skills/* \
    /etc/apparmor.d/usr.bin.hermes \
    /etc/apparmor.d/usr.bin.ollama \
"
