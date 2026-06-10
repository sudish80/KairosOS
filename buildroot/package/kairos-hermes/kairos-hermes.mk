################################################################################
#
# kairos-hermes
#
################################################################################

KAIROS_HERMES_VERSION = main
KAIROS_HERMES_SITE = https://github.com/NousResearch/hermes-agent.git
KAIROS_HERMES_SITE_METHOD = git
KAIROS_HERMES_DEPENDENCIES = python3 host-python3 nodejs git curl
KAIROS_HERMES_LICENSE = MIT

# Hermes is installed via pip in post-build, this package sets up
# the system dependencies and creates the kairos user environment.
# The actual agent installation happens in post-build.sh via pip.

define KAIROS_HERMES_INSTALL_TARGET_CMDS
	# Create deployment directories
	mkdir -p $(TARGET_DIR)/opt/kairos
	mkdir -p $(TARGET_DIR)/home/kairos/.hermes/skills
	mkdir -p $(TARGET_DIR)/home/kairos/.hermes/tools
	mkdir -p $(TARGET_DIR)/home/kairos/.hermes/memory
	mkdir -p $(TARGET_DIR)/home/kairos/.config
	mkdir -p $(TARGET_DIR)/etc/kairos

	# Install Kairos system tools
	$(INSTALL) -D -m 0755 $(KAIROS_HERMES_PKGDIR)/../../board/kairosos/rootfs_overlay/usr/bin/* \
		$(TARGET_DIR)/usr/bin/ 2>/dev/null || true

	# Install Kairos config
	$(INSTALL) -D -m 0644 $(KAIROS_HERMES_PKGDIR)/../../board/kairosos/rootfs_overlay/etc/kairos/* \
		$(TARGET_DIR)/etc/kairos/ 2>/dev/null || true
endef

$(eval $(generic-package))
