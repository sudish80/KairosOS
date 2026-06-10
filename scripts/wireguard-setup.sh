#!/bin/bash
# KairosOS WireGuard Configuration Generator
# Generates WireGuard config for the KairosOS mesh network.
set -euo pipefail

NODE_NAME="${1:-$(hostname)}"
MESH_NET="${2:-10.100.0}"
CONF_DIR="/etc/wireguard"
CONF_FILE="$CONF_DIR/kairos0.conf"

echo "KairosOS WireGuard Mesh Setup"
echo "  Node: $NODE_NAME"
echo "  Mesh: ${MESH_NET}.0/24"

mkdir -p "$CONF_DIR"

# Generate keys if not exist
PRIV_KEY="$CONF_DIR/kairos0.key"
PUB_KEY="$CONF_DIR/kairos0.key.pub"

if [ ! -f "$PRIV_KEY" ]; then
    wg genkey | tee "$PRIV_KEY" | wg pubkey > "$PUB_KEY"
    chmod 600 "$PRIV_KEY"
    echo "Generated WireGuard keys"
fi

PRIVATE=$(cat "$PRIV_KEY")
PUBLIC=$(cat "$PUB_KEY")

# Assign IP based on hostname hash
IP_OCTET=$(( $(echo "$NODE_NAME" | cksum | cut -d' ' -f1) % 253 + 2 ))
NODE_IP="${MESH_NET}.${IP_OCTET}"

# Generate config
cat > "$CONF_FILE" << EOF
# KairosOS WireGuard Mesh — Node: $NODE_NAME
[Interface]
Address = $NODE_IP/24
PrivateKey = $PRIVATE
ListenPort = 51820
MTU = 1420

# Enable kernel WireGuard
EOF

echo "Config written to $CONF_FILE"
echo "  Node IP: $NODE_IP/24"
echo "  Public key: $PUBLIC"
echo ""
echo "To bring up: wg-quick up $CONF_FILE"
echo "To add peers, append [Peer] sections with PublicKey, AllowedIPs, Endpoint."
