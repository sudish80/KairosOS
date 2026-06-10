// SPDX-License-Identifier: GPL-2.0
// KairosOS execsnoop — trace process execution via execve syscall
#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

char LICENSE[] SEC("license") = "GPL";

struct exec_event {
	__u32 pid;
	__u32 ppid;
	__u32 uid;
	__u32 ret;
	char comm[16];
	char filename[256];
};

struct {
	__uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
	__uint(key_size, sizeof(__u32));
	__uint(value_size, sizeof(__u32));
} exec_events SEC(".maps");

SEC("tracepoint/syscalls/sys_enter_execve")
int trace_execve_enter(struct trace_event_raw_sys_enter *ctx)
{
	struct exec_event event = {};
	__u64 pid_tgid = bpf_get_current_pid_tgid();
	event.pid = pid_tgid >> 32;
	event.ppid = bpf_get_current_pid_tgid() & 0xFFFFFFFF;
	event.uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
	bpf_get_current_comm(&event.comm, sizeof(event.comm));
	bpf_probe_read_user_str(&event.filename, sizeof(event.filename),
		(void *)ctx->args[0]);
	bpf_perf_event_output(ctx, &exec_events, BPF_F_CURRENT_CPU,
		&event, sizeof(event));
	return 0;
}

SEC("tracepoint/syscalls/sys_exit_execve")
int trace_execve_exit(struct trace_event_raw_sys_exit *ctx)
{
	struct exec_event event = {};
	__u64 pid_tgid = bpf_get_current_pid_tgid();
	event.pid = pid_tgid >> 32;
	event.ret = (__s32)ctx->ret;
	bpf_perf_event_output(ctx, &exec_events, BPF_F_CURRENT_CPU,
		&event, sizeof(event));
	return 0;
}
