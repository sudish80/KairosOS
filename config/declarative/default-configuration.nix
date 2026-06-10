# KairosOS Default Configuration
# Declarative system configuration — inspired by NixOS
# This file describes the entire OS state. AI agents read, edit,
# validate, and apply this config. Every change creates an atomic
# generation with full rollback capability.

{
  # === Agent Configuration ===
  agent = {
    # Primary AI assistant
    hermes = {
      enable = true;
      model = {
        provider = "ollama";
        name = "llama3.1:8b-q4_k_m";
        fallback = [
          { provider = "openai"; model = "gpt-4o"; }
          { provider = "openrouter"; model = "anthropic/claude-3.5-sonnet"; }
        ];
      };
      memory = {
        type = "knowledge-graph";
        vector_db = "sqlite-vec";
        auto_summarize = true;
        cross_session_graph = true;
      };
      skills = {
        auto_create = true;
        auto_improve = true;
        directories = [
          "/home/kairos/.hermes/skills"
          "/etc/kairos/skills"
        ];
      };
      gateway = {
        enable = false;
        telegram = { enable = false; bot_token = "@secret@"; };
        discord = { enable = false; bot_token = "@secret@"; };
        web = { enable = true; port = 8080; };
      };
    };

    # OpenClaw (optional multi-channel gateway)
    openclaw = {
      enable = false;
      channels = {
        telegram = { enable = false; };
        discord = { enable = false; };
        whatsapp = { enable = false; };
      };
    };
  };

  # === System Services ===
  services = {
    # SSH server
    sshd = {
      enable = true;
      port = 22;
      permit_root_login = false;
      password_authentication = false;
      allow_users = ["kairos"];
    };

    # Network time
    ntp = {
      enable = true;
      servers = ["0.pool.ntp.org" "1.pool.ntp.org"];
      fallback_ptp = true;
    };

    # Firewall
    firewall = {
      enable = true;
      default_policy = "drop";
      open_ports = [22 8080];
      allowed_interfaces = ["eth0" "wlan0"];
    };

    # Docker daemon
    docker = {
      enable = false;
      storage_driver = "overlay2";
      log_driver = "json-file";
      log_opts.max_size = "10m";
    };

    # Audio (for voice interaction)
    pipewire = {
      enable = true;
      low_latency = false;
    };

    # Avahi/mDNS (for mesh discovery)
    avahi = {
      enable = true;
      reflect = true;
    };
  };

  # === AI Services ===
  ai = {
    # Local LLM inference engine
    ollama = {
      enable = true;
      port = 11434;
      gpu = {
        enable = true;
        backend = "auto";  # cuda | rocm | vulkan | cpu
        memory_reserve = "2GB";
      };
      models = {
        default = "llama3.1:8b-q4_k_m";
        system_model = "kairos-system-1b:q4";
        cache_dir = "/opt/kairos/models";
        download_on_demand = true;
      };
    };

    # Personal Knowledge Graph
    knowledge_graph = {
      enable = true;
      storage = "/home/kairos/.hermes/knowledge.db";
      vector_extension = true;
      auto_extraction = true;
      nightly_consolidation = true;
      deterministic_deletion = true;
    };

    # eBPF telemetry
    ebpf_telemetry = {
      enable = true;
      programs = ["execsnoop" "tcptop" "filemon" "anomaly" "schedlatency" "oomkill"];
      anomaly_detection = true;
      auto_remediation = false;
    };
  };

  # === System Configuration ===
  system = {
    # Hostname
    hostname = "kairosos";

    # Kernel parameters
    kernel = {
      sysctl = {
        "vm.swappiness" = 10;
        "vm.vfs_cache_pressure" = 50;
        "vm.dirty_ratio" = 40;
        "vm.dirty_background_ratio" = 10;
        "net.core.default_qdisc" = "fq";
        "net.ipv4.tcp_congestion_control" = "bbr";
      };
      modules = [
        "kvm-intel" "kvm-amd" "wireguard" "overlay" "btrfs"
        "zram" "usbhid" "nvidia-current" "amdgpu"
      ];
    };

    # I/O scheduler
    io = {
      scheduler = "none";  # none (NVMe) | bfq (HDD) | kyber (mixed)
      nr_requests = 256;
    };

    # Power management
    power = {
      governor = "schedutil";
      energy_performance = "balance_performance";
      audio_power_save = true;
    };

    # Core isolation (for gaming/rendering)
    core_isolation = {
      enable = false;
      agent_cores = [0 1];
      user_cores = [2 3 4 5 6 7];
    };

    # OTA updates
    updates = {
      enable = true;
      channel = "stable";
      auto_check = true;
      auto_apply = false;
      mirror_world_verify = true;
    };

    # Declarative config
    config = {
      generations_to_keep = 10;
      auto_git_commit = true;
      commit_prefix = "kairos-auto";
    };
  };

  # === Users ===
  users = {
    kairos = {
      uid = 1000;
      group = "kairos";
      shell = "/bin/bash";
      home = "/home/kairos";
      extra_groups = ["docker" "wheel" "video" "audio"];
      sudo = "ALL=(ALL) NOPASSWD: /usr/bin/systemctl, /usr/bin/journalctl, /sbin/shutdown, /sbin/reboot";
    };
  };

  # === Hardware ===
  hardware = {
    gpu = {
      nvidia = { enable = false; driver = "nvidia-current"; };
      amd = { enable = false; driver = "amdgpu"; };
      intel = { enable = false; driver = "i915"; };
      enable_ollama_offload = true;
    };

    tpm = {
      enable = true;
      bind_agent_identity = true;
    };

    usbguard = {
      enable = true;
      policy = "block";
      allow_listed = ["046d:c52b" "04f2:b6ce"];
    };
  };
}
