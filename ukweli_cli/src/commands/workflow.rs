use crate::config::Config;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::path::Path;
use ukweli_db::workflow::Engine;

pub fn load<P: AsRef<Path>>(file: P) -> Result<()> {
    let file_path = file.as_ref();

    if !file_path.exists() {
        bail!("Workflow file not found: {}", file_path.display());
    }

    println!("Loading workflow from: {}", file_path.display());

    let content = std::fs::read_to_string(file_path).context("Failed to read workflow file")?;

    let workflow_json: Value = match file_path.extension().and_then(|s| s.to_str()) {
        Some("json") => serde_json::from_str(&content).context("Failed to parse JSON workflow")?,
        Some("yaml") | Some("yml") => {
            serde_yaml::from_str(&content).context("Failed to parse YAML workflow")?
        }
        _ => {
            bail!("Unsupported file format. Use .json, .yaml, or .yml");
        }
    };

    let mut engine = Engine::new();
    let workflow = engine
        .load_workflow_from_json(workflow_json.clone())
        .context("Workflow validation failed")?;

    println!("Workflow validated successfully");
    println!("ID:          {}", workflow.id);
    println!("Name:        {}", workflow.name);
    println!("Description: {}", workflow.description);
    println!("States:      {}", workflow.states.len());
    println!("Transitions: {}", workflow.transitions.len());

    let workflows_dir = Config::workflows_dir()?;
    std::fs::create_dir_all(&workflows_dir).context("Failed to create workflows directory")?;

    let workflow_file = workflows_dir.join(format!("{}.json", workflow.id));

    let json_str =
        serde_json::to_string_pretty(&workflow_json).context("Failed to serialize workflow")?;

    std::fs::write(&workflow_file, json_str).context("Failed to write workflow file")?;

    println!("Workflow saved to: {}", workflow_file.display());

    Ok(())
}
