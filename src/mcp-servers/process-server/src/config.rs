use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ps_path: String,
    pub kill_path: String,
    pub max_processes_returned: usize,
    pub allowed_signals: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ps_path: "/usr/bin/ps".into(),
            kill_path: "/usr/bin/kill".into(),
            max_processes_returned: 500,
            allowed_signals: vec![
                "TERM".into(),
                "KILL".into(),
                "HUP".into(),
                "USR1".into(),
                "USR2".into(),
            ],
        }
    }
}
