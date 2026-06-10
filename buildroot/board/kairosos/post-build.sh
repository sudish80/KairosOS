#!/bin/bash
# KairosOS post-build script
# Installs Hermes Agent and configures the Kairos environment

set -e

TARGET_DIR="${TARGET_DIR:-$1}"

if [ -z "$TARGET_DIR" ]; then
    echo "Usage: $0 <target-dir>"
    exit 1
fi

echo "KairosOS: Running post-build script..."

# Create kairos user if not exists
if ! grep -q "^kairos:" "$TARGET_DIR/etc/passwd" 2>/dev/null; then
    echo "kairos:x:1000:1000:KairosOS User:/home/kairos:/bin/bash" >> "$TARGET_DIR/etc/passwd"
    echo "kairos:x:1000:" >> "$TARGET_DIR/etc/group"
fi

# Set up sudo for kairos user
cat > "$TARGET_DIR/etc/sudoers.d/kairos" << 'EOF'
kairos ALL=(ALL) NOPASSWD: /usr/bin/systemctl, /usr/bin/journalctl, /usr/bin/apt, /usr/bin/pacman, /sbin/shutdown, /sbin/reboot
EOF
chmod 440 "$TARGET_DIR/etc/sudoers.d/kairos"

# Enable systemd services
SERVICES="kairos-agent kairos-terminal kairos-web"
for svc in $SERVICES; do
    if [ -f "$TARGET_DIR/etc/systemd/system/${svc}.service" ]; then
        ln -sf "/etc/systemd/system/${svc}.service" \
            "$TARGET_DIR/etc/systemd/system/multi-user.target.wants/${svc}.service"
    fi
done

# Install Hermes Agent via pip in target
# This is done at first boot via kairos-firstboot, but we set up the venv structure
mkdir -p "$TARGET_DIR/opt/hermes"

# Install first-boot script
cp "$TARGET_DIR/../../../scripts/first-boot.sh" "$TARGET_DIR/usr/sbin/kairos-firstboot"
chmod +x "$TARGET_DIR/usr/sbin/kairos-firstboot"

# Install first-boot systemd service
cp "$TARGET_DIR/../../../scripts/first-boot.service" "$TARGET_DIR/etc/systemd/system/kairos-firstboot.service"

ln -sf /etc/systemd/system/kairos-firstboot.service \
    "$TARGET_DIR/etc/systemd/system/multi-user.target.wants/kairos-firstboot.service"

# Copy Kairos system utilities
mkdir -p "$TARGET_DIR/usr/bin"
cat > "$TARGET_DIR/usr/bin/kairos-help" << 'KHELP'
#!/bin/bash
cat /usr/share/kairos/welcome.txt
echo ""
echo "Available commands:"
echo "  hermes             - Start interactive AI agent session"
echo "  hermes gateway     - Start multi-channel gateway (Telegram, Discord, etc.)"
echo "  kairos-status      - Show system status"
echo "  kairos-logs        - View AI agent logs"
echo "  kairos-update      - Update Hermes Agent"
echo "  kairos-config      - Edit agent configuration"
echo "  kairos-web         - Open web dashboard URL"
echo "  kairos-skill <name>- Create a new skill"
echo ""
echo "Say 'help' to the AI agent for AI-assisted system management."
KHELP
chmod +x "$TARGET_DIR/usr/bin/kairos-help"

cat > "$TARGET_DIR/usr/bin/kairos-status" << 'KSTAT'
#!/bin/bash
echo "╔═══════════════════════════════════════╗"
echo "║        KairosOS System Status         ║"
echo "╚═══════════════════════════════════════╝"
echo ""
echo "System: $(uname -a)"
echo "Uptime: $(uptime -p)"
echo ""
echo "--- Memory ---"
free -h
echo ""
echo "--- Disk ---"
df -h / /home 2>/dev/null
echo ""
echo "--- Agent ---"
if systemctl is-active --quiet kairos-agent 2>/dev/null; then
    echo "  AI Agent: Running"
else
    echo "  AI Agent: Stopped"
fi
if systemctl is-active --quiet kairos-web 2>/dev/null; then
    echo "  Web Dashboard: Running (http://localhost:8080)"
else
    echo "  Web Dashboard: Stopped"
fi
KSTAT
chmod +x "$TARGET_DIR/usr/bin/kairos-status"

cat > "$TARGET_DIR/usr/bin/kairos-logs" << 'KLOGS'
#!/bin/bash
journalctl -u kairos-agent -f -n 50 "$@"
KLOGS
chmod +x "$TARGET_DIR/usr/bin/kairos-logs"

cat > "$TARGET_DIR/usr/bin/kairos-config" << 'KCONF'
#!/bin/bash
EDITOR="${EDITOR:-nano}"
if [ -f /home/kairos/.hermes/config.yaml ]; then
    sudo -u kairos $EDITOR /home/kairos/.hermes/config.yaml
else
    echo "Config not found. Run kairos-firstboot or start the agent first."
fi
KCONF
chmod +x "$TARGET_DIR/usr/bin/kairos-config"

cat > "$TARGET_DIR/usr/bin/kairos-agent-check" << 'KCHECK'
#!/bin/bash
# Pre-startup check for the AI agent
if [ ! -f /home/kairos/.hermes/config.yaml ]; then
    echo "KairosOS: Agent not yet configured. Running first-boot setup..."
    /usr/sbin/kairos-firstboot
fi
exit 0
KCHECK
chmod +x "$TARGET_DIR/usr/bin/kairos-agent-check"

cat > "$TARGET_DIR/usr/bin/kairos-terminal-session" << 'KTERM'
#!/bin/bash
# Launch an interactive agent session in the terminal
echo "Starting KairosOS AI Agent session..."
echo "Say 'help' to see available commands or ask me anything."
echo ""
cd /home/kairos
sudo -u kairos /opt/hermes/bin/hermes
KTERM
chmod +x "$TARGET_DIR/usr/bin/kairos-terminal-session"

# Set ownership
chown -R 1000:1000 "$TARGET_DIR/home/kairos" 2>/dev/null || true

echo "KairosOS: Post-build complete."
