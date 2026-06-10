#!/bin/bash
# KairosOS System Hardening Script
# Run after first boot to apply security hardening.
set -euo pipefail

echo "KairosOS: Applying security hardening..."

# --- Kernel parameters ---
sysctl -w kernel.kptr_restrict=2
sysctl -w kernel.dmesg_restrict=1
sysctl -w kernel.printk=3 3 3 3
sysctl -w kernel.unprivileged_bpf_disabled=1
sysctl -w net.core.bpf_jit_harden=2
sysctl -w dev.tty.ldisc_autoload=0
sysctl -w vm.unprivileged_userfaultfd=0
sysctl -w kernel.kexec_load_disabled=1
sysctl -w kernel.sysrq=0
sysctl -w net.ipv4.tcp_syncookies=1
sysctl -w net.ipv4.tcp_rfc1337=1
sysctl -w net.ipv4.conf.all.rp_filter=1
sysctl -w net.ipv4.conf.default.rp_filter=1
sysctl -w net.ipv4.conf.all.accept_source_route=0
sysctl -w net.ipv4.conf.default.accept_source_route=0
sysctl -w net.ipv4.conf.all.accept_redirects=0
sysctl -w net.ipv4.conf.default.accept_redirects=0
sysctl -w net.ipv4.conf.all.secure_redirects=0
sysctl -w net.ipv4.conf.default.secure_redirects=0
sysctl -w net.ipv6.conf.all.accept_redirects=0
sysctl -w net.ipv6.conf.default.accept_redirects=0
sysctl -w net.ipv4.conf.all.send_redirects=0
sysctl -w net.ipv4.conf.default.send_redirects=0
sysctl -w net.ipv4.icmp_echo_ignore_broadcasts=1
sysctl -w net.ipv4.icmp_ignore_bogus_error_responses=1
sysctl -w net.ipv4.tcp_sack=0
sysctl -w net.ipv4.tcp_dsack=0
sysctl -w net.ipv4.tcp_fack=0
sysctl -w kernel.randomize_va_space=2

# Write to sysctl.d for persistence
cat > /etc/sysctl.d/99-kairos-hardening.conf << 'EOF'
kernel.kptr_restrict=2
kernel.dmesg_restrict=1
kernel.printk=3 3 3 3
kernel.unprivileged_bpf_disabled=1
net.core.bpf_jit_harden=2
dev.tty.ldisc_autoload=0
vm.unprivileged_userfaultfd=0
kernel.kexec_load_disabled=1
kernel.sysrq=0
net.ipv4.tcp_syncookies=1
net.ipv4.tcp_rfc1337=1
net.ipv4.conf.all.rp_filter=1
net.ipv4.conf.default.rp_filter=1
net.ipv4.conf.all.accept_source_route=0
net.ipv4.conf.default.accept_source_route=0
net.ipv4.conf.all.accept_redirects=0
net.ipv4.conf.default.accept_redirects=0
net.ipv4.conf.all.secure_redirects=0
net.ipv4.conf.default.secure_redirects=0
net.ipv6.conf.all.accept_redirects=0
net.ipv6.conf.default.accept_redirects=0
net.ipv4.conf.all.send_redirects=0
net.ipv4.conf.default.send_redirects=0
net.ipv4.icmp_echo_ignore_broadcasts=1
net.ipv4.icmp_ignore_bogus_error_responses=1
kernel.randomize_va_space=2
EOF

# --- File permissions ---
chmod 600 /etc/shadow
chmod 600 /etc/gshadow
chmod 644 /etc/passwd
chmod 644 /etc/group
chmod 750 /etc/sudoers.d
chmod 440 /etc/sudoers.d/kairos
chmod 700 /root
chmod 750 /home/kairos

# --- SSH hardening ---
if [ -f /etc/ssh/sshd_config ]; then
    sed -i 's/^#*PermitRootLogin.*/PermitRootLogin prohibit-password/' /etc/ssh/sshd_config
    sed -i 's/^#*PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config
    sed -i 's/^#*PubkeyAuthentication.*/PubkeyAuthentication yes/' /etc/ssh/sshd_config
    sed -i 's/^#*X11Forwarding.*/X11Forwarding no/' /etc/ssh/sshd_config
    sed -i 's/^#*MaxAuthTries.*/MaxAuthTries 3/' /etc/ssh/sshd_config
    sed -i 's/^#*ClientAliveInterval.*/ClientAliveInterval 300/' /etc/ssh/sshd_config
    sed -i 's/^#*ClientAliveCountMax.*/ClientAliveCountMax 2/' /etc/ssh/sshd_config
    sed -i 's/^#*Protocol.*/Protocol 2/' /etc/ssh/sshd_config
    systemctl restart sshd || true
fi

# --- AppArmor enforcement ---
if command -v aa-enforce &>/dev/null; then
    for profile in /etc/apparmor.d/usr.*; do
        if [ -f "$profile" ]; then
            aa-enforce "$profile" 2>/dev/null || true
        fi
    done
    systemctl reload apparmor 2>/dev/null || true
fi

# --- Firewall defaults ---
if command -v nft &>/dev/null; then
    cat > /etc/nftables.conf << 'EOF'
#!/usr/sbin/nft -f
table inet filter {
    chain input {
        type filter hook input priority filter; policy drop;
        ct state established,related accept
        iif lo accept
        ip protocol icmp accept
        tcp dport 22 accept
        tcp dport 11434 accept
        tcp dport 8080 accept
        counter drop
    }
    chain forward {
        type filter hook forward priority filter; policy drop;
    }
    chain output {
        type filter hook output priority filter; policy accept;
    }
}
EOF
    nft -f /etc/nftables.conf
    systemctl enable nftables 2>/dev/null || true
fi

# --- Remove dangerous compilers from non-root ---
chmod o-x /usr/bin/cc 2>/dev/null || true
chmod o-x /usr/bin/gcc 2>/dev/null || true
chmod o-x /usr/bin/ld 2>/dev/null || true

# --- Disable core dumps ---
cat > /etc/security/limits.d/99-kairos.conf << 'EOF'
* hard core 0
* soft core 0
EOF

echo "KairosOS: Hardening complete. Reboot recommended for kernel parameter changes."
