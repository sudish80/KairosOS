use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub entry: String,
    pub capabilities: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub enabled: bool,
    pub load_count: u64,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginLink {
    pub source: String,
    pub target: String,
    pub event: String,
    pub condition: Option<String>,
}

pub struct PluginEngine {
    config: Arc<RwLock<crate::config::Config>>,
    plugins: Arc<RwLock<HashMap<String, PluginInfo>>>,
    links: Arc<RwLock<Vec<PluginLink>>>,
    engine: wasmtime::Engine,
}

impl PluginEngine {
    pub fn new(config: Arc<RwLock<crate::config::Config>>) -> Self {
        let engine = wasmtime::Engine::default();
        Self {
            config,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            links: Arc::new(RwLock::new(Vec::new())),
            engine,
        }
    }

    pub async fn discover(&self, dir: &Path) -> anyhow::Result<Vec<PluginInfo>> {
        let mut found = Vec::new();
        let mut read_dir = fs::read_dir(dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("plugin.toml");
                if manifest_path.exists() {
                    match self.load_manifest(&manifest_path).await {
                        Ok(manifest) => {
                            let info = PluginInfo {
                                manifest,
                                path,
                                enabled: true,
                                load_count: 0,
                                last_error: None,
                            };
                            found.push(info.clone());
                            self.plugins
                                .write()
                                .await
                                .insert(info.manifest.name.clone(), info);
                        }
                        Err(e) => warn!(
                            "Failed to load plugin manifest at {:?}: {}",
                            manifest_path, e
                        ),
                    }
                }
            }
        }
        info!("Discovered {} plugins in {:?}", found.len(), dir);
        Ok(found)
    }

    pub async fn load_manifest(&self, path: &Path) -> anyhow::Result<PluginManifest> {
        let content = fs::read_to_string(path).await?;
        Ok(toml::from_str(&content)?)
    }

    pub async fn register_link(&self, link: PluginLink) {
        self.links.write().await.push(link);
    }

    pub async fn trigger_event(
        &self,
        event: &str,
        payload: serde_json::Value,
    ) -> anyhow::Result<()> {
        let links = self.links.read().await;
        let relevant: Vec<_> = links.iter().filter(|l| l.event == event).cloned().collect();
        drop(links);
        for link in relevant {
            if let Some(cond) = &link.condition {
                if !self.evaluate_condition(cond, &payload).await {
                    continue;
                }
            }
            self.execute_plugin(
                &link.source,
                "handle_event",
                serde_json::json!({
                    "event": event,
                    "payload": payload,
                }),
            )
            .await?;
        }
        Ok(())
    }

    async fn evaluate_condition(&self, _condition: &str, _payload: &serde_json::Value) -> bool {
        true
    }

    pub async fn execute_plugin(
        &self,
        name: &str,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let plugin = {
            let plugins = self.plugins.read().await;
            plugins.get(name).cloned()
        };
        match plugin {
            Some(mut info) => {
                info.load_count += 1;
                let wasm_path = info.path.join(&info.manifest.entry);
                if !wasm_path.exists() {
                    return Err(anyhow::anyhow!("WASM binary not found: {:?}", wasm_path));
                }
                match self.run_wasm(&wasm_path, method, &params).await {
                    Ok(result) => {
                        self.plugins.write().await.insert(name.to_string(), info);
                        Ok(result)
                    }
                    Err(e) => {
                        info.last_error = Some(e.to_string());
                        self.plugins.write().await.insert(name.to_string(), info);
                        Err(e)
                    }
                }
            }
            None => Err(anyhow::anyhow!("Plugin '{}' not found", name)),
        }
    }

    async fn run_wasm(
        &self,
        wasm_path: &Path,
        method: &str,
        params: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let module = wasmtime::Module::from_file(&self.engine, wasm_path)?;

        let mut linker = wasmtime::Linker::new(&self.engine);
        let mut store = wasmtime::Store::new(&self.engine, ());

        let input = serde_json::to_vec(&serde_json::json!({
            "method": method,
            "params": params,
        }))?;

        linker.func_wrap(
            "env",
            "log",
            |mut caller: wasmtime::Caller<'_, ()>, ptr: i32, len: i32| {
                let mem = match caller.get_export("memory") {
                    Some(wasmtime::Extern::Memory(m)) => m,
                    _ => return Err(wasmtime::Trap::new("no memory export")),
                };
                let data = mem.data(&caller);
                let msg = String::from_utf8_lossy(&data[ptr as usize..(ptr + len) as usize]);
                info!("[wasm:plugin] {}", msg);
                Ok(())
            },
        )?;

        linker.func_wrap(
            "env",
            "read_args",
            |mut caller: wasmtime::Caller<'_, ()>, ptr: i32, len: i32| {
                let mem = match caller.get_export("memory") {
                    Some(wasmtime::Extern::Memory(m)) => m,
                    _ => return Err(wasmtime::Trap::new("no memory export")),
                };
                let args = serde_json::to_vec(&input).unwrap_or_default();
                let data = mem.data_mut(&mut caller);
                let copy_len = args.len().min(len as usize);
                data[ptr as usize..(ptr as usize + copy_len)].copy_from_slice(&args[..copy_len]);
                Ok(args.len() as i32)
            },
        )?;

        let instance = linker.instantiate(&mut store, &module)?;

        let run = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "run")
            .map_err(|_| anyhow::anyhow!("WASM plugin missing 'run' export"))?;

        let result_ptr = run.call(&mut store, (input.len() as i32, 0))?;

        if result_ptr <= 0 {
            return Err(anyhow::anyhow!(
                "WASM plugin returned error code {}",
                result_ptr
            ));
        }

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("WASM module missing memory export"))?;

        let result_data = memory.data(&store)[result_ptr as usize..].to_vec();
        let result: serde_json::Value = serde_json::from_slice(&result_data).unwrap_or(
            serde_json::json!({"output": String::from_utf8_lossy(&result_data).to_string()}),
        );

        Ok(result)
    }

    pub async fn get_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.read().await.values().cloned().collect()
    }

    pub async fn get_plugin(&self, name: &str) -> Option<PluginInfo> {
        self.plugins.read().await.get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    #[tokio::test]
    async fn test_plugin_manifest_loading() {
        let dir = std::env::temp_dir().join("kairos_test_plugins");
        let _ = fs::remove_dir_all(&dir).await;
        fs::create_dir_all(&dir).await.unwrap();

        let manifest = PluginManifest {
            name: "test-plugin".into(),
            version: "1.0.0".into(),
            author: Some("test".into()),
            description: Some("test plugin".into()),
            entry: "plugin.wasm".into(),
            capabilities: vec!["test".into()],
            permissions: vec!["log".into()],
        };
        let manifest_path = dir.join("plugin.toml");
        fs::write(&manifest_path, toml::to_string(&manifest).unwrap())
            .await
            .unwrap();

        let cfg = Arc::new(RwLock::new(config::Config::default()));
        let engine = PluginEngine::new(cfg);
        let plugins = engine.discover(&dir).await.unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].manifest.name, "test-plugin");

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[test]
    fn test_manifest_serialization_roundtrip() {
        let manifest = PluginManifest {
            name: "net-monitor".into(),
            version: "0.2.0".into(),
            author: Some("kairos".into()),
            description: Some("Network monitoring plugin".into()),
            entry: "net_monitor.wasm".into(),
            capabilities: vec!["network:monitor".into(), "event:http".into()],
            permissions: vec!["log".into(), "http:fetch".into()],
        };
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let deserialized: PluginManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "net-monitor");
        assert_eq!(deserialized.capabilities.len(), 2);
    }

    #[test]
    fn test_plugin_link_serialization() {
        let link = PluginLink {
            source: "anomaly-detector".into(),
            target: "remediation".into(),
            event: "security:anomaly".into(),
            condition: Some("severity > 7".into()),
        };
        let json = serde_json::to_string(&link).unwrap();
        let back: PluginLink = serde_json::from_str(&json).unwrap();
        assert_eq!(back.event, "security:anomaly");
        assert_eq!(back.condition, Some("severity > 7".into()));
    }
}
