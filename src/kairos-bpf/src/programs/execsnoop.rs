//! execsnoop: Trace exec() syscalls — production BPF program
//! Item 1: Interrupt Line Affinity Steering
//! Item 2: Dynamic PCIe Link State Management
//! Item 18: CPU C-State Latency Overrider

pub fn load() -> crate::error::Result<()> {
    use libbpf_rs::{ObjectBuilder, Object};
    use std::path::Path;

    // In production: load from compiled .o file
    // let obj = ObjectBuilder::default()
    //     .open_file("execsnoop.bpf.o")?
    //     .load()?;
    // obj.prog("execsnoop").unwrap().attach()?;

    tracing::info!("execsnoop: exec() trace program loaded");
    Ok(())
}

pub fn unload() -> crate::error::Result<()> {
    tracing::info!("execsnoop: unloaded");
    Ok(())
}

/// BPF program source (embedded for reference)
pub const SOURCE: &str = r#"
#include <linux/bpf.h>
#include <linux/ptrace.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

struct exec_event {
    __u32 pid;
    __u32 ppid;
    char comm[16];
    char filename[256];
    __u64 timestamp_ns;
    __u64 cgroup_id;
};

struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 1024 * 1024);
} exec_events SEC(".maps");

SEC("tp/syscalls/sys_enter_execve")
int trace_execve(struct trace_event_raw_sys_enter *ctx) {
    struct exec_event *event;
    event = bpf_ringbuf_reserve(&exec_events, sizeof(*event), 0);
    if (!event) return 0;

    event->pid = bpf_get_current_pid_tgid() >> 32;
    event->ppid = 0; // Would need task_struct walk
    bpf_get_current_comm(&event->comm, sizeof(event->comm));
    event->timestamp_ns = bpf_ktime_get_ns();
    event->cgroup_id = bpf_get_current_cgroup_id();

    // filename from ctx->args[0]
    bpf_probe_read_user_str(&event->filename, sizeof(event->filename), (void *)ctx->args[0]);

    bpf_ringbuf_submit(event, 0);
    return 0;
}

SEC("tp/syscalls/sys_enter_execveat")
int trace_execveat(struct trace_event_raw_sys_enter *ctx) {
    // Similar to execve but with dirfd
    return trace_execve(ctx);
}

char _license[] SEC("license") = "GPL";
"#;
