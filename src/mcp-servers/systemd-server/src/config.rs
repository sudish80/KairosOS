use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub systemctl_path: String,
    pub journalctl_path: String,
    pub default_journal_lines: u64,
    pub service_actions: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            systemctl_path: "/usr/bin/systemctl".into(),
            journalctl_path: "/usr/bin/journalctl".into(),
            default_journal_lines: 50,
            service_actions: vec![
                "start".into(),
                "stop".into(),
                "restart".into(),
                "enable".into(),
                "disable".into(),
            ],
        }
    }
}
