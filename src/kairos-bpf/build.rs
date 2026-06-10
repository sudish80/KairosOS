// Build script for kairos-bpf: compiles eBPF programs
use std::path::PathBuf;

fn main() {
    // Tell cargo to re-run if BPF source changes
    println!("cargo:rerun-if-changed=src/programs/");

    // Compile BPF programs if feature enabled
    if std::env::var("CARGO_FEATURE_BPF").is_ok() {
        compile_bpf_programs();
    }
}

fn compile_bpf_programs() {
    let bpf_dir = PathBuf::from("bpf");
    if !bpf_dir.exists() {
        eprintln!("BPF source directory not found: {:?}", bpf_dir);
        return;
    }

    // In production, use libbpf-cargo to compile
    // For now, just verify clang is available
    if std::process::Command::new("clang")
        .args(["--version"])
        .output()
        .is_err()
    {
        eprintln!("clang not found - BPF compilation skipped");
        return;
    }

    // Compile each BPF program
    let programs = [
        "execsnoop",
        "tcptop",
        "filemon",
        "anomaly",
        "schedlatency",
        "oomkill",
    ];
    for prog in programs {
        let src = bpf_dir.join(format!("{}.bpf.c", prog));
        let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join(format!("{}.o", prog));

        if src.exists() {
            let status = std::process::Command::new("clang")
                .args([
                    "-target",
                    "bpf",
                    "-O2",
                    "-g",
                    "-Wall",
                    "-Wno-unused-value",
                    "-Wno-pointer-sign",
                    "-Wno-compare-distinct-pointer-types",
                    "-I",
                    "/usr/include",
                    "-I",
                    "/usr/include/linux",
                    "-c",
                    src.to_str().unwrap(),
                    "-o",
                    out.to_str().unwrap(),
                ])
                .status();

            match status {
                Ok(s) if s.success() => println!("Compiled BPF: {}", prog),
                Ok(s) => eprintln!(
                    "BPF compilation failed for {}: exit code {:?}",
                    prog,
                    s.code()
                ),
                Err(e) => eprintln!("Failed to run clang for {}: {}", prog, e),
            }
        }
    }
}
