#!/bin/bash
# KairosOS Installer — transforms an existing Linux system into KairosOS
# Usage: curl -fsSL https://kairosos.org/install.sh | sh
#        curl -fsSL https://kairosos.org/install.sh | sh -s -- --unstable

set -euo pipefail

VERSION="0.1.0"
REPO="https://github.com/your-org/kairosos"

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RESET='\033[0m'

log_info()  { echo -e "${CYAN}⏺${RESET} $1"; }
log_ok()    { echo -e "${GREEN}✓${RESET} $1"; }
log_warn()  { echo -e "${YELLOW}⚠${RESET} $1"; }
log_error() { echo -e "${RED}✗${RESET} $1"; }

# --- Prerequisites ---

check_prereqs() {
    log_info "Checking prerequisites..."
    local missing=()

    for cmd in curl git python3 node pip3 systemctl; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing+=("$cmd")
        fi
    done

    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Missing required commands: ${missing[*]}"
        log_info "Install them with your package manager first."
        exit 1
    fi

    log_ok "All prerequisites met."
}

# --- Install Hermes Agent ---

install_hermes() {
    log_info "Installing Hermes Agent..."
    if command -v hermes >/dev/null 2>&1; then
        log_ok "Hermes Agent already installed."
        return
    fi
    curl -fsSL https://hermes-agent.nousresearch.com/install.sh | bash
    log_ok "Hermes Agent installed."
}

# --- Create KairosOS Config & Services ---

setup_kairos() {
    log_info "Setting up KairosOS..."

    # Create kairos user
    if ! id -u kairos >/dev/null 2>&1; then
        sudo useradd -m -s /bin/bash -c "KairosOS User" kairos
        log_ok "Created kairos user."
    fi

    # Create directories
    sudo mkdir -p /opt/kairos/web
    sudo mkdir -p /etc/kairos
    sudo mkdir -p /var/log/kairos

    # Install systemd services
    local SERVICES_DIR="$(dirname "$0")/../buildroot/board/kairosos/rootfs_overlay/etc/systemd/system"
    if [ -d "$SERVICES_DIR" ]; then
        sudo cp "$SERVICES_DIR"/*.service /etc/systemd/system/
        sudo systemctl daemon-reload
        sudo systemctl enable kairos-agent kairos-web
        log_ok "Installed systemd services."
    else
        # Create services inline
        create_services_inline
    fi

    # Copy skills
    local SKILLS_DIR="$(dirname "$0")/../agent/skills"
    if [ -d "$SKILLS_DIR" ]; then
        sudo mkdir -p /etc/kairos/skills
        sudo cp "$SKILLS_DIR"/*.md /etc/kairos/skills/
        log_ok "Installed KairosOS skills."
    fi

    log_ok "KairosOS setup complete."
}

create_services_inline() {
    # Minimal service creation for standalone install
    cat > /tmp/kairos-agent.service << 'SVC'
[Unit]
Description=KairosOS AI Agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=kairos
ExecStart=/home/kairos/.local/bin/hermes
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
SVC
    sudo mv /tmp/kairos-agent.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable kairos-agent
}

# --- Configure Hermes for KairosOS ---

configure_hermes() {
    log_info "Configuring Hermes Agent for KairosOS..."

    local HERMES_CONFIG="/home/kairos/.hermes/config.yaml"

    if [ -f "$HERMES_CONFIG" ]; then
        log_warn "Hermes config already exists. Backing up..."
        sudo cp "$HERMES_CONFIG" "${HERMES_CONFIG}.bak"
    fi

    sudo mkdir -p /home/kairos/.hermes/skills /home/kairos/.hermes/tools /home/kairos/.hermes/memory

    # Copy KairosOS skills to user directory
    if [ -d /etc/kairos/skills ]; then
        sudo cp /etc/kairos/skills/*.md /home/kairos/.hermes/skills/
    fi

    sudo chown -R kairos:kairos /home/kairos/.hermes

    log_ok "Hermes Agent configured for KairosOS."
}

# --- Print status ---

print_status() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${RESET}"
    echo -e "${GREEN}║       KairosOS Installation Complete   ║${RESET}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${RESET}"
    echo ""
    echo "  Start the AI agent:  sudo systemctl start kairos-agent"
    echo "  Check status:        systemctl status kairos-agent"
    echo "  Agent logs:          journalctl -u kairos-agent -f"
    echo "  Chat with Kairos:    hermes"
    echo "  Web dashboard:       http://localhost:8080"
    echo ""
    echo "  Say 'help' to the agent to see available system commands."
    echo ""
}

# --- Main ---

main() {
    echo ""
    echo -e "${CYAN}   ╔══════════════════════════════════════╗${RESET}"
    echo -e "${CYAN}   ║         KairosOS Installer ${VERSION}      ║${RESET}"
    echo -e "${CYAN}   ╚══════════════════════════════════════╝${RESET}"
    echo ""

    if [ "$(id -u)" -eq 0 ]; then
        log_warn "Running as root. Creating non-root kairos user."
    fi

    check_prereqs
    install_hermes
    setup_kairos
    configure_hermes
    print_status
}

main "$@"
