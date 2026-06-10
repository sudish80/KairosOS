# KairosOS shared configuration class
# Inherited by all KairosOS recipes

inherit systemd

KAIROS_DAEMON_DIR ?= "/usr/sbin"
KAIROS_CONF_DIR ?= "/etc/kairos"
KAIROS_DATA_DIR ?= "/var/lib/kairos"
KAIROS_LOG_DIR ?= "/var/log/kairos"
KAIROS_SOCK_DIR ?= "/run/kairos"

# All KaiROS daemons are Rust binaries
DEPENDS:append = " cargo-native"

do_compile() {
    oe_cargo_build
}

do_install:append() {
    # Create standard Kairos directories
    install -d ${D}${KAIROS_CONF_DIR}
    install -d ${D}${KAIROS_DATA_DIR}
    install -d ${D}${KAIROS_LOG_DIR}
    install -d ${D}${KAIROS_SOCK_DIR}
}

FILES:${PN} += " \
    ${KAIROS_CONF_DIR} \
    ${KAIROS_DATA_DIR} \
    ${KAIROS_LOG_DIR} \
    ${KAIROS_SOCK_DIR} \
"
