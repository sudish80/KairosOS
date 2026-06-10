# Service Manager Skill

## Description
Manage systemd services, units, timers, and system state. Start, stop, enable, disable, and monitor services.

## Triggers
- User asks about service status
- Service failure detected
- Boot optimization requests

## Actions

### list(status)
List services filtered by status: running | stopped | failed | all

### status(service)
Show detailed status of a specific service

### start(service)
Start a service

### stop(service)
Stop a service

### restart(service)
Restart a service

### enable(service)
Enable a service to start at boot

### disable(service)
Disable a service from starting at boot

### logs(service, lines, follow)
View service logs via journalctl

### analyze_boot()
Show boot time analysis (systemd-analyze)

### list_timers()
List all systemd timers

## Example
User: "My web server stopped working"
Agent: *checks service status, views logs, finds error, fixes config, restarts*

## Dependencies
- systemctl
- journalctl
- systemd-analyze
