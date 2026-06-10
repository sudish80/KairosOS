require recipes-kernel/linux/linux-yocto.inc

DESCRIPTION = "KairosOS custom Linux kernel with full eBPF, IOMMU, TEE support"
LICENSE = "GPL-2.0-only"
LIC_FILES_CHKSUM = "file://COPYING;md5=6bc538ed5bd9a7fc9398086aedcd7e46"

KERNEL_VERSION_SANITY_SKIP = "1"

LINUX_VERSION = "6.12"
LINUX_VERSION_EXTENSION = "-kairosos"

SRC_URI = " \
    git://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git;protocol=https;branch=linux-6.12.y \
    file://kairosos-v2.config \
"

SRCREV = "v6.12"

# Kernel config
KERNEL_CONFIG_DIR = "${LAYERDIR}/../../config/kernel"
SRC_URI += "file://kairosos-v2.config"

do_configure:prepend() {
    cp ${WORKDIR}/kairosos-v2.config ${B}/.config
}

# Required kernel modules for KairosOS
KERNEL_MODULE_AUTOLOAD:append = " \
    kvm-amd \
    kvm-intel \
    nftables \
    wireguard \
    btrfs \
    zram \
"

# Enable kernel features for KairosOS
KERNEL_FEATURES:append = " \
    features/bpf/bpf.scc \
    features/io_uring/io_uring.scc \
    features/kvm/kvm.scc \
    features/security/security.scc \
"

COMPATIBLE_MACHINE = "qemux86-64|qemuarm64|genericx86-64"
