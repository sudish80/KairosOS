use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEndpoint {
    pub method: String,
    pub path: String,
    pub description: String,
    pub parameters: Vec<ParameterSchema>,
    pub response_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    pub name: String,
    pub kind: String,
    pub optional: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSpec {
    pub name: String,
    pub version: String,
    pub description: String,
    pub endpoints: Vec<LiveEndpoint>,
    pub dependencies: Vec<String>,
    pub config_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureDocument {
    pub version: String,
    pub generated_at: f64,
    pub daemons: HashMap<String, DaemonSpec>,
    pub data_flows: Vec<DataFlow>,
    pub dependencies: Vec<DependencyEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlow {
    pub source: String,
    pub target: String,
    pub protocol: String,
    pub data_type: String,
    pub frequency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
}

pub struct LiveArchitecture {
    daemons: Arc<RwLock<HashMap<String, DaemonSpec>>>,
    data_flows: Arc<RwLock<Vec<DataFlow>>>,
    output_path: String,
    listen_port: u16,
}

impl LiveArchitecture {
    pub fn new(output_path: &str, listen_port: u16) -> Self {
        Self {
            daemons: Arc::new(RwLock::new(HashMap::new())),
            data_flows: Arc::new(RwLock::new(Vec::new())),
            output_path: output_path.to_string(),
            listen_port,
        }
    }

    pub async fn register_daemon(&self, spec: DaemonSpec) {
        self.daemons.write().await.insert(spec.name.clone(), spec);
    }

    pub async fn register_flow(&self, flow: DataFlow) {
        self.data_flows.write().await.push(flow);
    }

    pub async fn generate_architecture_doc(&self) -> ArchitectureDocument {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let daemons = self.daemons.read().await.clone();
        let data_flows = self.data_flows.read().await.clone();

        let dependencies: Vec<DependencyEdge> = data_flows.iter().map(|f| {
            DependencyEdge {
                from: f.source.clone(),
                to: f.target.clone(),
                kind: format!("{}/{}", f.protocol, f.data_type),
            }
        }).collect();

        ArchitectureDocument {
            version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: now,
            daemons,
            data_flows,
            dependencies,
        }
    }

    pub async fn export_json(&self) -> anyhow::Result<String> {
        let doc = self.generate_architecture_doc().await;
        let json = serde_json::to_string_pretty(&doc)?;
        Ok(json)
    }

    pub async fn export_mermaid(&self) -> String {
        let daemons = self.daemons.read().await;
        let flows = self.data_flows.read().await;

        let mut mermaid = String::from("graph TD\n");
        for daemon in daemons.keys() {
            mermaid.push_str(&format!("    {}[{}]\n", daemon.replace("-", ""), daemon));
        }
        for flow in flows.iter() {
            let src = flow.source.replace("-", "");
            let tgt = flow.target.replace("-", "");
            mermaid.push_str(&format!(
                "    {}-->|{}:{}|{}\n",
                src, flow.protocol, flow.data_type, tgt
            ));
        }
        mermaid
    }

    pub async fn export_plantuml(&self) -> String {
        let daemons = self.daemons.read().await;
        let flows = self.data_flows.read().await;

        let mut plant = String::from("@startuml\nskinparam componentStyle rectangle\n");
        for daemon in daemons.keys() {
            plant.push_str(&format!("component \"{}\" as {}\n", daemon, daemon.replace("-", "")));
        }
        for flow in flows.iter() {
            let src = flow.source.replace("-", "");
            let tgt = flow.target.replace("-", "");
            plant.push_str(&format!("{} -->> {} : {}/{} ({}ms)\n",
                src, tgt, flow.protocol, flow.data_type, flow.frequency_ms));
        }
        plant.push_str("@enduml\n");
        plant
    }

    pub async fn generate_svg(&self) -> anyhow::Result<Vec<u8>> {
        let mermaid = self.export_mermaid().await;

        let tmpfile = std::env::temp_dir().join("kairos-arch.mmd");
        fs::write(&tmpfile, &mermaid).await?;

        let output = tokio::process::Command::new("npx")
            .args(["-y", "@mermaid-js/mermaid-cli", "-i"])
            .arg(&tmpfile)
            .args(["-o", &std::env::temp_dir().join("kairos-arch.svg").to_string_lossy()])
            .output()
            .await?;

        if output.status.success() {
            let svg = fs::read(std::env::temp_dir().join("kairos-arch.svg")).await?;
            Ok(svg)
        } else {
            Ok(mermaid.into_bytes())
        }
    }

    pub async fn save_documentation(&self) -> anyhow::Result<()> {
        let json = self.export_json().await?;
        let mermaid = self.export_mermaid();
        let plantuml = self.export_plantuml();

        std::fs::create_dir_all(&self.output_path)?;

        fs::write(format!("{}/architecture.json", self.output_path), &json).await?;
        fs::write(format!("{}/architecture.mmd", self.output_path), &mermaid).await?;
        fs::write(format!("{}/architecture.puml", self.output_path), &plantuml).await?;

        info!("Architecture documentation saved to {}", self.output_path);
        Ok(())
    }

    pub async fn start_http_endpoint(&self) -> anyhow::Result<()> {
        let daemons = Arc::clone(&self.daemons);
        let flows = Arc::clone(&self.data_flows);
        let output = self.output_path.clone();

        let app = warp::path("api")
            .and(warp::path("v1"))
            .and(warp::path("architecture"))
            .and(warp::path("json"))
            .map(move || {
                let daemons = daemons.blocking_read();
                let data_flows = flows.blocking_read();
                let doc = ArchitectureDocument {
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    generated_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64(),
                    daemons: daemons.clone(),
                    data_flows: data_flows.clone(),
                    dependencies: data_flows.iter().map(|f| DependencyEdge {
                        from: f.source.clone(), to: f.target.clone(),
                        kind: format!("{}/{}", f.protocol, f.data_type),
                    }).collect(),
                };
                warp::reply::json(&doc)
            });

        info!("Live architecture docs on http://0.0.0.0:{}/api/v1/architecture/json", self.listen_port);
        warp::serve(app)
            .run(([0, 0, 0, 0], self.listen_port))
            .await;

        Ok(())
    }
}

impl Default for DaemonSpec {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: "0.1.0".into(),
            description: String::new(),
            endpoints: Vec::new(),
            dependencies: Vec::new(),
            config_keys: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_generate() {
        let arch = LiveArchitecture::new("/tmp/kairos-arch-test", 0);
        let spec = DaemonSpec {
            name: "kairos-bpf".into(),
            version: "1.0.0".into(),
            description: "eBPF telemetry".into(),
            endpoints: vec![LiveEndpoint {
                method: "GET".into(),
                path: "/api/v1/bpf/metrics".into(),
                description: "Get BPF metrics".into(),
                parameters: vec![],
                response_type: "JSON".into(),
            }],
            dependencies: vec!["kairos-db".into()],
            config_keys: vec!["bpf.poll_interval".into()],
        };
        arch.register_daemon(spec).await;
        arch.register_flow(DataFlow {
            source: "kairos-bpf".into(),
            target: "kairos-db".into(),
            protocol: "unix-socket".into(),
            data_type: "binary".into(),
            frequency_ms: 1000,
        }).await;

        let doc = arch.generate_architecture_doc().await;
        assert!(doc.daemons.contains_key("kairos-bpf"));
        assert_eq!(doc.data_flows.len(), 1);
    }

    #[tokio::test]
    async fn test_mermaid_export() {
        let arch = LiveArchitecture::new("/tmp", 0);
        arch.register_daemon(DaemonSpec {
            name: "kairos-mcp".into(), ..Default::default()
        }).await;
        arch.register_daemon(DaemonSpec {
            name: "kairos-db".into(), ..Default::default()
        }).await;
        arch.register_flow(DataFlow {
            source: "kairos-mcp".into(),
            target: "kairos-db".into(),
            protocol: "tcp".into(),
            data_type: "json".into(),
            frequency_ms: 500,
        }).await;

        let mermaid = arch.export_mermaid();
        assert!(mermaid.contains("kaikros-mcp"));
        assert!(mermaid.contains("kaikros-db"));
    }

    #[tokio::test]
    async fn test_plantuml_export() {
        let arch = LiveArchitecture::new("/tmp", 0);
        arch.register_daemon(DaemonSpec {
            name: "kairos-llm".into(), ..Default::default()
        }).await;
        arch.register_flow(DataFlow {
            source: "kairos-llm".into(),
            target: "kairos-orchestrator".into(),
            protocol: "unix".into(),
            data_type: "proto".into(),
            frequency_ms: 100,
        }).await;

        let plant = arch.export_plantuml();
        assert!(plant.contains("kairos-llm"));
        assert!(plant.contains("kairos-orchestrator"));
    }

    #[test]
    fn test_endpoint_serialization() {
        let ep = LiveEndpoint {
            method: "POST".into(),
            path: "/api/v1/apply".into(),
            description: "Apply config".into(),
            parameters: vec![ParameterSchema {
                name: "target".into(),
                kind: "string".into(),
                optional: false,
                description: "Target daemon".into(),
            }],
            response_type: "JSON".into(),
        };
        let json = serde_json::to_string(&ep).unwrap();
        let back: LiveEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(back.method, "POST");
        assert_eq!(back.path, "/api/v1/apply");
    }
}
