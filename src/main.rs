mod config;

use anyhow::{bail, Result};
use config::{load_config, BedrockServer, NodeJs};
use std::{path::Path, process::Stdio};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config("./Config.toml").await?;

    let BedrockServer { entry_path } = config.bedrock_server;
    let entry = Path::new(&entry_path);
    if !entry.is_dir() {
        bail!("[Config.toml] Invalid bedrock_server path. Type the path to the folder containing bds.")
    }
    let mut bedrock_cmd = Command::new(format!("{}/bedrock_server", entry_path));
    if cfg!(target_os = "linux") {
        bedrock_cmd.env("LD_LIBRARY_PATH", entry_path);
    }
    let mut bedrock_cmd = bedrock_cmd.stdout(Stdio::piped()).spawn().unwrap();
    let mut bedrock_stdout = BufReader::new(bedrock_cmd.stdout.take().unwrap()).lines();

    let NodeJs {
        entry_path,
        main_module,
    } = config.node_js_script;
    let entry = Path::new(&entry_path);
    if !entry.is_dir() {
        bail!("[Config.toml] Invalid node_js path. Type the path to the folder containing main_module.")
    } else if !entry.join(&main_module).exists() {
        bail!(format!("[Config.toml] '{}' doesn't exist.", main_module))
    }
    let mut node_cmd = Command::new("node")
        .stdout(Stdio::piped())
        .arg(format!("{}/{}", entry_path, main_module))
        .spawn()
        .unwrap();
    let mut node_stdout = BufReader::new(node_cmd.stdout.take().unwrap()).lines();

    tokio::spawn(async move {
        while let Ok(Some(text)) = node_stdout.next_line().await {
            println!("{}", text);
        }
    });

    let _ = tokio::spawn(async move {
        while let Ok(Some(text)) = bedrock_stdout.next_line().await {
            println!("{}", text);
            if &text[..] == "Quit correctly" {
                println!("[Wrapper] Server Stopped.");
                node_cmd.kill().await.unwrap();
                node_cmd.wait().await.unwrap();
                bedrock_cmd.wait().await.unwrap();
                break;
            }
        }
    })
    .await;

    Ok(())
}
