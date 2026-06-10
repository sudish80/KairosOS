# BPF Telemetry Skill

## Description
Leverage eBPF-powered telemetry from the kairos-bpf daemon for deep system observability — real-time process execution, TCP throughput, file I/O latency, scheduling latency, and OOM monitoring.

## Triggers
- User asks about real-time system events or performance
- Security or anomaly detection events
- Automated diagnostics

## Actions

### watch_exec()
Stream new process executions via execsnoop BPF probe:
- PID, PPID, command, arguments, timestamp
- Filter by user, binary name, or arguments

### watch_tcp(port=None)
Monitor TCP connection throughput via tcptop BPF probe:
- Source/dest IP:port, bytes sent/received, duration
- Filter by port, process, or CIDR range

### watch_file(path=None)
Trace file I/O operations via filemon BPF probe:
- PID, file path, operation (read/write/open/close), latency
- Filter by path prefix or process name

### watch_sched()
Monitor scheduler latency via schedlatency BPF probe:
- Run queue latency per-task, CPU migration events
- Wakeup preemption tracking

### watch_oom()
Stream OOM killer events via oomkill BPF probe:
- Triggering cgroup, victim PID/name, memory usage at death
- Total RSS, swap, and OOM score

### anomaly_scan(duration=60)
Run the anomaly BPF probe across all event sources:
- Detect outlier behavior: unexpected execs, traffic spikes, file I/O storms
- Return ranked anomalies with severity

### get_bpf_stats()
Query kairos-bpf daemon health:
- Active probes, total events captured, dropped events
- Per-probe memory usage, uptime

## Example
User: "Show me all processes started in the last 30 seconds"
Agent: *calls watch_exec with streaming filter, returns process table*

## Dependencies
- kairos-bpf daemon (MCP endpoint)
- Linux kernel with eBPF (CONFIG_BPF, CONFIG_BPF_EVENTS)
- /sys/kernel/debug/tracing (tracefs)
