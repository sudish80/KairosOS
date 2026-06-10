# Building KairosOS

## Prerequisites

- **Docker** (recommended) — builds in a containerized environment
- Or a **Linux build machine** with:
  - Build essentials (gcc, make, etc.)
  - Python 3.11+
  - Node.js 18+
  - 8GB+ RAM
  - 20GB+ free disk

## Build Methods

### 1. Docker Build (Recommended)

```bash
# Clone the repository
git clone https://github.com/your-org/kairosos.git
cd kairosos

# Build the ISO (this handles everything)
make build

# Or use the build script directly
./scripts/build.sh
```

The build will:
1. Build a Docker image with all build dependencies
2. Fetch Buildroot source code
3. Configure and build the Linux kernel
4. Build the root filesystem with all packages
5. Install Hermes Agent and KairosOS tools
6. Generate a bootable ISO

### 2. Direct Build (Linux Only)

```bash
# Install Buildroot dependencies (Debian/Ubuntu)
sudo apt-get install -y \
    bash bc binutils bison build-essential bzr ca-certificates \
    cmake cpio curl debianutils file flex g++ gcc git gperf \
    graphviz kmod libelf-dev libncurses-dev libssl-dev lz4 make \
    openssl patch perl pkg-config python3 python3-pip python3-dev \
    rsync sed tar unzip wget xz-utils zlib1g-dev zstd

# Fetch Buildroot
git clone --depth 1 --branch 2026.02 \
    https://github.com/buildroot/buildroot.git buildroot-src

# Build
cd buildroot-src
make BR2_EXTERNAL=/path/to/kairosos/buildroot kairosos_defconfig
make BR2_EXTERNAL=/path/to/kairosos/buildroot
```

### 3. Install on Existing Linux

For systems already running Linux, use the installer:

```bash
# One-liner
curl -fsSL https://kairosos.org/install.sh | sh

# Or from the repo
./scripts/install.sh
```

This will:
- Install Hermes Agent
- Create systemd services
- Install KairosOS skills
- Configure the agent for system access

## Output Artifacts

After a successful build, you'll find these in `output/images/`:

| File | Description |
|------|-------------|
| `kairosos.iso` | Bootable ISO image |
| `rootfs.ext4` | Root filesystem image |
| `bzImage` | Linux kernel binary |
| `*.dtb` | Device tree blobs (ARM) |

## Running in QEMU

```bash
# Boot from ISO
qemu-system-x86_64 -m 2048 -cdrom output/images/kairosos.iso -boot d

# Boot with direct kernel
qemu-system-x86_64 -m 2048 -smp 2 \
    -drive file=output/images/rootfs.ext4,format=raw,if=virtio \
    -kernel output/images/bzImage \
    -append "root=/dev/vda console=ttyS0" \
    -nographic
```

## First Boot

On first boot, KairosOS will:
1. Run the first-boot setup script
2. Install Hermes Agent (if not pre-installed)
3. Start the AI agent service
4. Show the welcome screen on tty1

Login:
- Username: `kairos`
- Password: `kairos` (change on first login)

## Customization

### Changing the Kernel Config

```bash
make linux-menuconfig    # In buildroot
```

### Adding Packages

Edit `kairosos_defconfig` and add:
```
BR2_PACKAGE_YOUR_PACKAGE=y
```

### Custom Skills

Add `.md` files to `buildroot/board/kairosos/rootfs_overlay/etc/kairos/skills/`
