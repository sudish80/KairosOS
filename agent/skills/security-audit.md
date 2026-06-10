# Security Audit Skill

## Description
Scan the system for security issues, check configurations, and harden the OS.

## Triggers
- User asks about security
- Scheduled security scans
- Suspicious activity detected

## Actions

### quick_audit()
Run a quick security scan:
- Check for failed SSH login attempts
- List open ports
- Check for available security updates
- Check user accounts with sudo access
- Check firewall status

### full_audit()
Comprehensive security audit:
- All quick_audit checks
- Check file permissions on sensitive files (/etc/shadow, /etc/sudoers, etc.)
- Check for world-writable files
- Check for SUID/SGID binaries
- Audit user accounts and groups
- Check SSH configuration
- Check kernel security parameters (sysctl)
- Audit running services
- Check for suspicious cron jobs
- Check systemd service sandboxing

### harden()
Apply security hardening:
- Harden SSH configuration (key-only auth, disable root login)
- Set kernel security parameters
- Configure firewall defaults
- Set file permissions
- Enable automatic security updates
- Configure fail2ban (if available)

### check_logs(pattern)
Search system logs for security events matching a pattern

## Example
User: "Is my system secure?"
Agent: *runs quick_audit, finds 3 issues, fixes them, reports results*

## Dependencies
- journalctl
- iptables/nftables
- Sudo access
