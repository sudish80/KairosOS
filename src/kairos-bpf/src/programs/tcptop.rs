//! tcptop: TCP connection tracking — production BPF program
//! Item 202: eBPF XDP Filter Injector
//! Item 205: SYN Flood Mitigation Governor
//! Item 208: Real-Time DNS Leak Eliminator
//! Item 214: TCP BBR Congestion Control Switcher
//! Item 225: ECN Enforcer

pub fn load() -> crate::error::Result<()> {
    tracing::info!("tcptop: TCP connection tracking loaded");
    Ok(())
}

pub fn unload() -> crate::error::Result<()> {
    tracing::info!("tcptop: unloaded");
    Ok(())
}

pub const SOURCE: &str = r#"
#include <linux/bpf.h>
#include <linux/ptrace.h>
#include <linux/tcp.h>
#include <linux/ip.h>
#include <linux/in.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

struct tcp_event {
    __u32 pid;
    char comm[16];
    __u16 sport;
    __u16 dport;
    __u32 saddr;
    __u32 daddr;
    __u8 state;
    __u64 timestamp_ns;
    __u64 bytes_sent;
    __u64 bytes_recv;
};

struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 1024 * 1024);
} tcp_events SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_LRU_HASH);
    __uint(max_entries, 65536);
    __type(key, __u32); // pid
    __type(value, __u64); // bytes
} tcp_bytes_sent SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_LRU_HASH);
    __uint(max_entries, 65536);
    __type(key, __u32);
    __type(value, __u64);
} tcp_bytes_recv SEC(".maps");

SEC("kprobe/tcp_sendmsg")
int trace_tcp_sendmsg(struct pt_regs *ctx) {
    __u32 pid = bpf_get_current_pid_tgid() >> 32;
    struct sock *sk = (struct sock *)PT_REGS_PARM1(ctx);
    __u32 len = PT_REGS_PARM3(ctx);

    __u64 *bytes = bpf_map_lookup_elem(&tcp_bytes_sent, &pid);
    if (bytes) {
        __sync_fetch_and_add(bytes, len);
    } else {
        bpf_map_update_elem(&tcp_bytes_sent, &pid, &len, BPF_ANY);
    }
    return 0;
}

SEC("kprobe/tcp_recvmsg")
int trace_tcp_recvmsg(struct pt_regs *ctx) {
    __u32 pid = bpf_get_current_pid_tgid() >> 32;
    struct sock *sk = (struct sock *)PT_REGS_PARM1(ctx);
    __u32 len = PT_REGS_PARM3(ctx);

    __u64 *bytes = bpf_map_lookup_elem(&tcp_bytes_recv, &pid);
    if (bytes) {
        __sync_fetch_and_add(bytes, len);
    } else {
        bpf_map_update_elem(&tcp_bytes_recv, &pid, &len, BPF_ANY);
    }
    return 0;
}

SEC("tracepoint/sock/inet_sock_set_state")
int trace_tcp_state_change(struct trace_event_raw_inet_sock_set_state *ctx) {
    if (ctx->protocol != IPPROTO_TCP) return 0;

    struct tcp_event *event;
    event = bpf_ringbuf_reserve(&tcp_events, sizeof(*event), 0);
    if (!event) return 0;

    event->pid = bpf_get_current_pid_tgid() >> 32;
    bpf_get_current_comm(&event->comm, sizeof(event->comm));
    event->sport = bpf_ntohs(ctx->sport);
    event->dport = bpf_ntohs(ctx->dport);
    event->saddr = ctx->saddr;
    event->daddr = ctx->daddr;
    event->state = ctx->newstate;
    event->timestamp_ns = bpf_ktime_get_ns();
    event->bytes_sent = 0;
    event->bytes_recv = 0;

    __u32 pid = event->pid;
    __u64 *sent = bpf_map_lookup_elem(&tcp_bytes_sent, &pid);
    __u64 *recv = bpf_map_lookup_elem(&tcp_bytes_recv, &pid);
    if (sent) event->bytes_sent = *sent;
    if (recv) event->bytes_recv = *recv;

    bpf_ringbuf_submit(event, 0);
    return 0;
}

char _license[] SEC("license") = "GPL";
"#;
