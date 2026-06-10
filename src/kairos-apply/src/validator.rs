//! Config validation with schema checking and rule enforcement
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config;
use crate::parser::DeclarativeConfig;
use crate::error::ApplyError;

pub struct ConfigValidator {
    config: Arc<RwLock<config::Config>>,
    rules: Vec<ValidationRule>,
}

pub enum Severity { Error, Warning }

pub struct ValidationRule {
    pub name: &'static str,
    pub check: Box<dyn Fn(&DeclarativeConfig) -> Result<(), String> + Send + Sync>,
    pub severity: Severity,
}

impl ConfigValidator {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        let mut v = Self { config, rules: Vec::new() };
        v.register_default_rules();
        v
    }

    fn register_default_rules(&mut self) {
        self.rules.push(ValidationRule {
            name: "version_check",
            check: Box::new(|c| {
                if c.version.is_empty() { Err("Config version must not be empty".into()) }
                else { Ok(()) }
            }),
            severity: Severity::Error,
        });
        self.rules.push(ValidationRule {
            name: "metadata_check",
            check: Box::new(|c| {
                if c.metadata.name.is_empty() { Err("Config name must not be empty".into()) }
                else { Ok(()) }
            }),
            severity: Severity::Error,
        });
        self.rules.push(ValidationRule {
            name: "path_no_abs",
            check: Box::new(|c| {
                for (_, spec) in &c.files {
                    if !spec.path.starts_with('/') {
                        return Err(format!("Path must be absolute: {}", spec.path));
                    }
                }
                Ok(())
            }),
            severity: Severity::Error,
        });
    }

    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }

    pub fn validate(&self, config: &DeclarativeConfig) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for rule in &self.rules {
            if let Err(msg) = (rule.check)(config) {
                errors.push(format!("[Validation] {}: {}", rule.name, msg));
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
