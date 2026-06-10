// Rollback verification and execution

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const GENERATIONS_DIR: &str = "/etc/kairos/generations";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackCheck {
    pub can_rollback: bool,
    pub reason: String,
    pub target_generation: Option<String>,
}

pub fn verify(target_id: &str) -> Result<RollbackCheck, String> {
    let gen_path = PathBuf::from(GENERATIONS_DIR).join(target_id);

    if !gen_path.exists() {
        return Ok(RollbackCheck {
            can_rollback: false,
            reason: format!("Generation '{}' not found", target_id),
            target_generation: None,
        });
    }

    let active_path = gen_path.join("config.json");
    if !active_path.exists() {
        return Ok(RollbackCheck {
            can_rollback: false,
            reason: "Generation has no configuration file".into(),
            target_generation: None,
        });
    }

    Ok(RollbackCheck {
        can_rollback: true,
        reason: "Ready for rollback".into(),
        target_generation: Some(target_id.into()),
    })
}
