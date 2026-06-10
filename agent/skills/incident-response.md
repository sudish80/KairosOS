# Incident Response Skill

## Description
Handle security and system incidents — containment, forensics, recovery, and reporting. Integrates with eBPF telemetry, AppArmor, auditd, and the knowledge graph for full incident lifecycle management.

## Triggers
- Suspicious activity detected by eBPF anomaly probe
- AppArmor denial events
- Failed SSH or auth attempts exceeding threshold
- User reports a security concern

## Actions

### triage(incident_id=None)
Evaluate current or specified incident:
- Severity assessment (low/medium/high/critical)
- Affected services, users, and files
- Timeline of events from audit log and eBPF telemetry
- Recommended immediate actions

### contain(incident_id, strategy)
Isolate the threat:
- `strategy`: isolate_process | block_ip | disable_user | quarantine_path
- Uses nftables, systemd scopes, or AppArmor
- Returns containment result and residual access paths

### investigate(incident_id, depth="standard")
Deep forensic investigation:
- `depth`: quick | standard | full
- Collects process trees, network connections, file access logs
- Correlates with knowledge graph for entity matching
- Captures memory snapshot (if full depth)
- Generates timeline visualization

### remediate(incident_id, strategy)
Apply remediation:
- `strategy`: revert | clean | restore | rebuild
- Revert: rollback config generation
- Clean: remove malicious files/users
- Restore: restore from backup
- Rebuild: rebuild affected service/container

### generate_report(incident_id, format="markdown")
Create incident report:
- `format`: markdown | json | html
- Includes timeline, affected assets, actions taken, evidence
- Stores report in knowledge graph for future reference

### watch_threats(duration=300)
Active threat monitoring session:
- Streams security-relevant events from eBPF and auditd
- Real-time severity scoring
- Auto-contains if threshold exceeded

## Example
User: "We have an incident — unknown SSH access detected"
Agent: *calls triage to assess, contain to block source IP, investigate for forensics*

## Dependencies
- kairos-bpf daemon (anomaly probe)
- AppArmor (audit logs)
- auditd
- nftables
- kairos-pkg (knowledge graph storage)
- kairos-apply (remediation via rollback)
