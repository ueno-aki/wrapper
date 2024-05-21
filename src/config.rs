use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;

pub async fn load_config(path: &str) -> Result<Config> {
    let config_str = fs::read_to_string(path).await?;
    toml::from_str(&config_str).map_err(|e| anyhow!("Invalid config.toml: {e:?}"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bedrock_server: BedrockServer,
    pub node_js_script: NodeJs,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BedrockServer {
    pub entry_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeJs {
    pub entry_path: String,
    pub main_module: String,
}
