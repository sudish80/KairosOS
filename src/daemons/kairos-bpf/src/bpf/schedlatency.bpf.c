// SPDX-License-Identifier: GPL-2.0
// KairosOS schedlatency — measure scheduler wakeup latency
#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

char LICENSE[] SEC("license") = "GPL";

struct sched_event {
	__u32 pid;
	__u32 prev_pid;
	__u64 wakeup_latency_ns;
	char comm[16];
	char prev_comm[16];
};

struct {
	__uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
	__uint(key_size, sizeof(__u32));
	__uint(value_size, sizeof(__u32));
} sched_events SEC(".maps");

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 16384);
	__type(key, __u32);
	__type(value, __u64);
} wakeup_timestamps SEC(".maps");

SEC("tp_btf/sched_wakeup")
int BPF_PROG(trace_sched_wakeup, struct task_struct *p)
{
	__u32 pid;
	__u64 ts;

	bpf_core_read(&pid, sizeof(pid), &p->pid);
	ts = bpf_ktime_get_ns();
	bpf_map_update_elem(&wakeup_timestamps, &pid, &ts, BPF_ANY);
	return 0;
}

SEC("tp_btf/sched_switch")
int BPF_PROG(trace_sched_switch, bool preempt, struct task_struct *prev, struct task_struct *next)
{
	__u32 pid;
	__u64 *wakeup_ts, delta;

	bpf_core_read(&pid, sizeof(pid), &next->pid);

	wakeup_ts = bpf_map_lookup_elem(&wakeup_timestamps, &pid);
	if (wakeup_ts) {
		delta = bpf_ktime_get_ns() - *wakeup_ts;
		if (delta > 0 && delta < 1000000000) {
			struct sched_event event = {};
			event.pid = pid;
			event.wakeup_latency_ns = delta;
			bpf_core_read(&event.comm, sizeof(event.comm), &next->comm);
			bpf_core_read(&event.prev_pid, sizeof(event.prev_pid), &prev->pid);
			bpf_core_read(&event.prev_comm, sizeof(event.prev_comm), &prev->comm);
			bpf_perf_event_output(ctx, &sched_events, BPF_F_CURRENT_CPU,
				&event, sizeof(event));
		}
		bpf_map_delete_elem(&wakeup_timestamps, &pid);
	}
	return 0;
}
