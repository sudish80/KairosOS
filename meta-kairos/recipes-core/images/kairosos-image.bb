DESCRIPTION = "KairosOS — AI-native Linux operating system image"
LICENSE = "MIT"

inherit core-image

# Image features
IMAGE_FEATURES += " \
    ssh-server-openssh \
    package-management \
    hwcodecs \
    debug-tweaks \
    allow-empty-password \
"

# System base
IMAGE_INSTALL:append = " \
    packagegroup-core-boot \
    packagegroup-core-buildessential \
    packagegroup-base \
    kernel-modules \
    linux-firmware \
    openssh \
    systemd \
    systemd-analyze \
    udev \
    util-linux \
    e2fsprogs \
    btrfs-progs \
    dosfstools \
    gptfdisk \
    grub \
"

# Networking
IMAGE_INSTALL:append = " \
    iptables \
    nftables \
    wireguard-tools \
    dhcpcd \
    networkmanager \
    iproute2 \
    bridge-utils \
    tcpdump \
    traceroute \
    mtr \
"

# Monitoring & debugging
IMAGE_INSTALL:append = " \
    strace \
    ltrace \
    lsof \
    htop \
    iotop \
    iperf3 \
    sysstat \
    perf \
    bpftool \
    valgrind \
    gdb \
"

# Security
IMAGE_INSTALL:append = " \
    apparmor \
    apparmor-profiles \
    apparmor-utils \
    auditd \
    fail2ban \
    libseccomp \
    libcap \
    checksec \
"

# KairosOS daemons
IMAGE_INSTALL:append = " \
    kairos-bpf \
    kairos-mcp \
    kairos-conf \
    kairos-git-logger \
    kairos-pkg \
"

# AI/ML
IMAGE_INSTALL:append = " \
    python3 \
    python3-pip \
    python3-numpy \
    python3-pyyaml \
    ollama \
"

# Development tools
IMAGE_INSTALL:append = " \
    git \
    vim \
    tmux \
    curl \
    wget \
    jq \
    yq \
"

# User services
IMAGE_INSTALL:append = " \
    pipewire \
    pipewire-pulse \
    avahi \
    docker \
    docker-compose \
"

# Size optimizations
IMAGE_ROOTFS_EXTRA_SPACE = "4194304"
IMAGE_OVERHEAD_FACTOR = "1.3"

# Systemd as init
INIT_MANAGER = "systemd"
