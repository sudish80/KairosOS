# System Monitor Skill

## Description
Monitor system health, performance, and resource usage. Provides real-time metrics, historical analysis, and proactive alerting.

## Triggers
- User asks about system performance or health
- Automated health check intervals
- Anomaly detection events

## Actions

### get_system_overview
Returns a summary of system state:
- CPU: load average, per-core usage, temperature
- Memory: total, used, available, swap
- Disk: mount points, usage, inodes
- Network: interfaces, throughput, errors
- Uptime and load

### monitor_resource(name, duration)
Watch a specific resource over time:
- `name`: cpu | memory | disk | network | io
- `duration`: time period in seconds (default: 60)

### check_health
Run comprehensive health checks:
- Filesystem mount health
- Network connectivity
- Service status
- Disk SMART data (if available)
- Certificate expiry checks

### set_alert(condition, action)
Configure proactive alerting:
- `condition`: cpu > 90% | memory > 85% | disk > 90% | service_down
- `action`: log | notify | auto_remediate

## Example
User: "Why is my system slow?"
Agent: *runs get_system_overview, identifies high memory usage, suggests solutions*

## Dependencies
- /proc filesystem
- /sys filesystem
- systemd journal
- htop, iostat, vmstat (if available)
