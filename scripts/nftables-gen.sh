#!/bin/bash
# KairosOS nftables Firewall Config Generator
# Generates a comprehensive nftables ruleset for KairosOS.
set -euo pipefail

OUTPUT="${1:-/etc/nftables.conf}"

cat > "$OUTPUT" << 'EOF'
#!/usr/sbin/nft -f

# KairosOS AI-Native Firewall
# Auto-generated — customize per deployment

flush ruleset

table inet kairos {
    # Connection tracking
    set allowed_ports {
        type inet_service; flags interval;
        elements = { 22, 80, 443, 11434, 8080, 51820 }
    }

    set trusted_subnets {
        type ipv4_addr; flags interval;
        elements = { 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16 }
    }

    # Rate limiting
    set ratelimit_v4 {
        type ipv4_addr;
        size 65536;
        flags dynamic, timeout;
        timeout 60s;
    }

    chain input {
        type filter hook input priority filter; policy drop;

        # Allow established traffic
        ct state established,related accept

        # Loopback
        iif lo accept

        # Drop invalid
        ct state invalid drop

        # ICMP
        ip protocol icmp limit rate 10/second accept
        ip6 nexthdr icmpv6 accept

        # Rate limit SSH
        tcp dport 22 add @ratelimit_v4 { ip saddr limit rate 3/minute burst 5 packets } accept

        # Allowed services
        tcp dport @allowed_ports accept

        # MCP internal socket
        tcp dport 8237 ip saddr @trusted_subnets accept

        # Log and drop
        log prefix "KAIROS-DROP: " limit rate 5/minute burst 10 packets
        counter drop
    }

    chain forward {
        type filter hook forward priority filter; policy drop;
        ct state established,related accept
        iif docker0 oif docker0 accept
        counter drop
    }

    chain output {
        type filter hook output priority filter; policy accept;
    }
}

table inet kairos_nat {
    chain postrouting {
        type nat hook postrouting priority srcnat; policy accept;
        oifname "eth0" masquerade
    }
}
EOF

chmod 600 "$OUTPUT"
echo "Generated nftables config: $OUTPUT"
echo "Apply with: nft -f $OUTPUT"
