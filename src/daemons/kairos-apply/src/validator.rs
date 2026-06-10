// Configuration validator — checks for correctness and consistency

use crate::parser::ParsedConfig;

pub fn validate(config: &ParsedConfig) -> Vec<String> {
    let mut issues = Vec::new();

    // Validate agent config
    if config.agent.hermes.is_none() && config.agent.openclaw.is_none() {
        issues.push("At least one AI agent (Hermes or OpenClaw) must be enabled".into());
    }

    // Validate hardware config
    if let Some(hw) = &config.hardware.gpu {
        let gpu_count = hw.as_object()
            .map(|o| o.values().filter(|v| v.as_bool().unwrap_or(false)).count())
            .unwrap_or(0);
        if gpu_count > 1 {
            issues.push("Multiple GPU drivers enabled — only one should be active".into());
        }
    }

    // Validate system config
    if let Some(sys) = &config.system.hostname {
        if sys.len() > 63 {
            issues.push("Hostname must be 63 characters or less".into());
        }
        if !sys.chars().all(|c| c.is_alphanumeric() || c == '-') {
            issues.push("Hostname must be alphanumeric with hyphens only".into());
        }
    }

    // Validate firewall config
    if let Some(fw) = &config.services.firewall {
        if let Some(ports) = fw.as_object().and_then(|o| o.get("open_ports")) {
            if let Some(port_list) = ports.as_array() {
                for port in port_list {
                    match port.as_u64() {
                        Some(p) if p > 65535 => issues.push(format!("Invalid port: {}", p)),
                        _ => {}
                    }
                }
            }
        }
    }

    // Validate OTA settings
    if let Some(updates) = &config.system.updates {
        if let Some(channel) = updates.as_object().and_then(|o| o.get("channel")) {
            match channel.as_str() {
                Some("stable") | Some("beta") | Some("dev") => {}
                Some(other) => issues.push(format!("Unknown update channel: {}", other)),
                None => {}
            }
        }
    }

    issues
}
