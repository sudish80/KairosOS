# Update Manager Skill

## Description
Control system updates via the KairosOS declarative config engine and OTA update mechanism. Supports A/B partition updates, generation-based rollback, and staged rollout.

## Triggers
- User asks about updates or upgrades
- Scheduled update check intervals
- Security advisory received

## Actions

### check_updates()
Check for available updates:
- Kernel, initramfs, system daemons, user packages
- Returns version diff and severity ratings
- Checks update server or local cache

### list_generations()
List all declarative config generations:
- Generation ID, timestamp, description
- SHA256 content hash, active/booted flags
- Rollback availability

### apply_update(description=None)
Create a new generation from current system state:
- Captures /etc/kairos/configuration.nix state
- Creates atomic generation with SHA256 content hash
- Validates configuration before applying
- Updates bootloader for A/B slot management

### rollback(generation_id=None)
Rollback to a previous generation:
- Uses kairos-apply rollback engine
- Default: rollback to previous generation
- Validates rollback target compatibility

### status()
Show update system status:
- Current generation, booted generation
- Update channel (stable/beta/nightly)
- Last update check timestamp
- Pending updates

### set_channel(channel)
Set update channel:
- `channel`: stable | beta | nightly
- Controls which update stream to track
- Persists in declarative config

### schedule_check(interval_hours=24)
Schedule automatic update checks:
- Creates systemd timer
- Optionally auto-apply with rollback threshold

## Example
User: "Update the system"
Agent: *calls check_updates, shows available updates, calls apply_update*

## Dependencies
- kairos-apply daemon (MCP endpoint)
- kairos-git-logger daemon
- systemd (timer units)
- A/B update partition scheme
