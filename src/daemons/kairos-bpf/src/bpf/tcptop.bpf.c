// SPDX-License-Identifier: GPL-2.0
// KairosOS tcptop — trace TCP connections by PID
#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

char LICENSE[] SEC("license") = "GPL";

struct connect_event {
	__u32 pid;
	__u32 uid;
	__u32 saddr;
	__u32 daddr;
	__u16 sport;
	__u16 dport;
	int ret;
	char comm[16];
};

struct {
	__uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
	__uint(key_size, sizeof(__u32));
	__uint(value_size, sizeof(__u32));
} connect_events SEC(".maps");

SEC("kprobe/tcp_v4_connect")
int trace_connect(struct pt_regs *ctx)
{
	struct connect_event event = {};
	__u64 pid_tgid = bpf_get_current_pid_tgid();
	event.pid = pid_tgid >> 32;
	event.uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
	bpf_get_current_comm(&event.comm, sizeof(event.comm));
	bpf_perf_event_output(ctx, &connect_events, BPF_F_CURRENT_CPU,
		&event, sizeof(event));
	return 0;
}
