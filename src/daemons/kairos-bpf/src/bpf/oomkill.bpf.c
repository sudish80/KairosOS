// SPDX-License-Identifier: GPL-2.0
// KairosOS oomkill — detect OOM killer events before they happen
#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

char LICENSE[] SEC("license") = "GPL";

struct oom_event {
	__u32 pid;
	__u32 oom_score;
	__u64 rss_anon;
	__u64 rss_file;
	__u64 rss_shmem;
	char comm[16];
	char cgroup_path[128];
};

struct {
	__uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
	__uint(key_size, sizeof(__u32));
	__uint(value_size, sizeof(__u32));
} oom_events SEC(".maps");

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 1024);
	__type(key, __u32);
	__type(value, struct oom_event);
} oom_predictions SEC(".maps");

SEC("tracepoint/events/oom/oom_mark_victim")
int trace_oom_mark_victim(struct trace_event_raw_oom_mark_victim *ctx)
{
	struct oom_event event = {};
	__u64 pid_tgid = bpf_get_current_pid_tgid();
	event.pid = ctx->pid;
	event.oom_score = ctx->oom_score;
	bpf_get_current_comm(&event.comm, sizeof(event.comm));
	bpf_perf_event_output(ctx, &oom_events, BPF_F_CURRENT_CPU,
		&event, sizeof(event));
	return 0;
}
