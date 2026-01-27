use crate::config::Config;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::path::Path;
use ukweli_db::{Workflow, workflow::Engine};

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

pub fn list() -> Result<()> {
    // list all loaded workflows
    let workflows_dir = Config::workflows_dir()?;

    if !workflows_dir.exists() {
        println!("No workflows loaded.");
        println!("Load a workflow with: ukweli workflow load <file>");
        return Ok(());
    }

    let mut workflows = Vec::new();

    for entry in std::fs::read_dir(&workflows_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            match load_workflow_from_file(&path) {
                Ok(workflow) => workflows.push(workflow),
                Err(e) => {
                    eprintln!("Failed to load {}: {}", path.display(), e);
                }
            }
        }
    }

    if workflows.is_empty() {
        println!("No workflows found.");
        return Ok(());
    }

    println!("Loaded workflows ({}):\n", workflows.len());

    for workflow in workflows {
        println!("{}", workflow.name);
        println!("ID:          {}", workflow.id);
        println!("Description: {}", workflow.description);
        println!("States:      {}", workflow.states.len());
        println!("Transitions: {}", workflow.transitions.len());
    }

    Ok(())
}

fn load_workflow_from_file<P: AsRef<Path>>(path: P) -> Result<Workflow> {
    let content = std::fs::read_to_string(path.as_ref()).context("Failed to read workflow file")?;

    let workflow_json: Value =
        serde_json::from_str(&content).context("Failed to parse workflow JSON")?;

    let mut engine = Engine::new();
    let workflow = engine
        .load_workflow_from_json(workflow_json)
        .context("Failed to load workflow")?;

    Ok(workflow)
}
