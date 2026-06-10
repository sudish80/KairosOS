# KairosOS User Guide

## First Time

When you boot KairosOS for the first time, you'll be greeted by the Kairos
welcome screen on tty1. The AI agent starts automatically.

**Login**: `kairos` / `kairos` (change immediately!)

## Talking to Kairos

### Terminal (TUI)

```bash
hermes
```

This launches the full Hermes Agent TUI with:
- Multiline editing
- Slash-command autocomplete
- Streaming tool output
- Conversation history

### Web Dashboard

Open http://localhost:8080 in a browser for a chat interface
with system monitoring.

### From Anywhere (Gateway)

Set up messaging gateways:

```bash
hermes gateway setup
hermes gateway start
```

Supports: Telegram, Discord, Slack, WhatsApp, Signal, and more.

## System Management Examples

### System Health
```
You: How's my system doing?
Kairos: Shows CPU, memory, disk, and network overview
```

### Install Software
```
You: Install nginx and set it up with HTTPS
Kairos: Installs nginx, configures SSL, opens firewall, starts service
```

### Network Troubleshooting
```
You: My internet is slow, can you check?
Kairos: Runs diagnostics, finds issue, suggests fix
```

### Security Audit
```
You: Check my system security
Kairos: Runs audit, reports findings, offers to fix
```

### Cleanup
```
You: Free up disk space
Kairos: Finds large files, cleans caches, removes temp data
```

### Logs
```
You: Show me the last 50 errors from the system log
Kairos: Checks journalctl and summarizes issues
```

## Built-in Commands

| Command | Description |
|---------|-------------|
| `kairos-help` | Show help information |
| `kairos-status` | System status dashboard |
| `kairos-logs` | Follow agent logs |
| `kairos-config` | Edit agent configuration |
| `kairos-update` | Update Hermes Agent |

## Agent Slash Commands (inside Hermes)

| Command | Description |
|---------|-------------|
| `/new` or `/reset` | Start fresh conversation |
| `/model` | Change AI model |
| `/personality` | Set agent personality |
| `/retry` | Retry last response |
| `/undo` | Undo last action |
| `/compress` | Compress conversation context |
| `/usage` | Check token usage |
| `/skills` | Browse installed skills |
| `/memory` | Search memories |

## Skills System

KairosOS comes with pre-installed skills for system management.
The agent can also create new skills automatically after complex tasks.

```bash
# List skills
/skills

# Use a skill directly
/system-monitor
/package-manager install nginx
```

Skills live in `~/.hermes/skills/` and `/etc/kairos/skills/`.

## Memory System

Kairos remembers across conversations:

```bash
# Search memories
/memory "nginx config"

# View memory stats
/insights
```

## OpenClaw Integration

If you also want OpenClaw's multi-channel gateway:

```bash
# Install OpenClaw
npm install -g openclaw@latest

# Configure
openclaw onboard
```

KairosOS includes migration tools if you're switching from OpenClaw:

```bash
hermes claw migrate
```

## Troubleshooting

### Agent won't start
```bash
sudo systemctl status kairos-agent
journalctl -u kairos-agent -n 50 --no-pager
kairos-firstboot  # Re-run setup
```

### Network issues
```bash
kairos: "Check my network"
# or manually:
ip a
ping -c 4 8.8.8.8
```

### Reset configuration
```bash
rm -rf ~/.hermes
sudo systemctl restart kairos-agent
```
