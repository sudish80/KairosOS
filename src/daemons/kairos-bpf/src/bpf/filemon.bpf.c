// SPDX-License-Identifier: GPL-2.0
// KairosOS filemon — trace file open/write/unlink operations
#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

char LICENSE[] SEC("license") = "GPL";

struct file_event {
	__u32 pid;
	__u32 uid;
	__u32 flags;
	int ret;
	char comm[16];
	char filename[256];
};

struct {
	__uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
	__uint(key_size, sizeof(__u32));
	__uint(value_size, sizeof(__u32));
} file_events SEC(".maps");

SEC("tracepoint/syscalls/sys_enter_openat")
int trace_openat_enter(struct trace_event_raw_sys_enter *ctx)
{
	struct file_event event = {};
	__u64 pid_tgid = bpf_get_current_pid_tgid();
	event.pid = pid_tgid >> 32;
	event.flags = (__u32)ctx->args[1];
	bpf_get_current_comm(&event.comm, sizeof(event.comm));
	bpf_probe_read_user_str(&event.filename, sizeof(event.filename),
		(void *)ctx->args[0]);
	bpf_perf_event_output(ctx, &file_events, BPF_F_CURRENT_CPU,
		&event, sizeof(event));
	return 0;
}
