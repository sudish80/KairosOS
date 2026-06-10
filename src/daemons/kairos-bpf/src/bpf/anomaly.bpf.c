// SPDX-License-Identifier: GPL-2.0
// KairosOS anomaly — detect unusual syscall patterns (security sensor)
#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

char LICENSE[] SEC("license") = "GPL";

#define ANOMALY_UNKNOWN_SYSCALL 0xFFFF

struct anomaly_event {
	__u32 pid;
	__u32 syscall_nr;
	__u32 uid;
	__u32 ret;
	char comm[16];
};

struct {
	__uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
	__uint(key_size, sizeof(__u32));
	__uint(value_size, sizeof(__u32));
} anomaly_events SEC(".maps");

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 1024);
	__type(key, __u32);
	__type(value, __u32);
} allowed_syscalls SEC(".maps");

SEC("tracepoint/raw_syscalls/sys_enter")
int trace_anomaly(struct trace_event_raw_sys_enter *ctx)
{
	__u32 syscall_nr = (__u32)ctx->id;
	__u32 *allowed;
	__u32 pid;

	allowed = bpf_map_lookup_elem(&allowed_syscalls, &syscall_nr);
	if (allowed) {
		return 0;
	}

	pid = bpf_get_current_pid_tgid() >> 32;

	struct anomaly_event event = {};
	event.pid = pid;
	event.syscall_nr = syscall_nr;
	event.uid = bpf_get_current_uid_gid();
	bpf_get_current_comm(&event.comm, sizeof(event.comm));
	bpf_perf_event_output(ctx, &anomaly_events, BPF_F_CURRENT_CPU,
		&event, sizeof(event));
	return 0;
}
