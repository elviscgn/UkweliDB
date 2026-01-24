use anyhow::{Context, Result};
use std::path::PathBuf;
use ukweli_db::{Ledger, storage::writer::DatabaseWriter};

use crate::config::Config;

pub fn run(db_path: Option<PathBuf>) -> Result<()> {
    println!("Initialising Ukweli database...\n");
    println!("Path {:?}", db_path);

    let mut config = Config::load_or_default()?;

    if let Some(custom_path) = db_path {
        config.db_path = custom_path;
    }

    if config.db_path.exists() {
        anyhow::bail!("Database already exists at: {}", config.db_path.display())
    }

    // add the workflow/user & .ukweli folders
    create_directory_structure()?;

    println!("Setting up genesis ledger...");
    let ledger = Ledger::new();

    println!("Writing up ledger to: {}", config.db_path.display());

    if let Some(parent) = config.db_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create database directory")?;
    }

    let mut writer =
        DatabaseWriter::new(&config.db_path).context("Failed to create database writer")?;

    writer
        .write_ledger(&ledger)
        .context("Failed to write initial ledger")?;

    println!("Note: GENESIS user is in the ledger but cannot sign new records from CLI");
    println!("Create new users with: ukweli user create <username>");
    println!("Initialisation complete!");

    Ok(())
}

fn create_directory_structure() -> Result<()> {
    println!("Setting up directory...");

    let ukweli_dir = Config::ukweli_dir()?;
    let users_dir = Config::users_dir()?;
    let workflows_dir = Config::workflows_dir()?;

    std::fs::create_dir_all(&ukweli_dir).context("Failed to create .ukweli directory")?;

    std::fs::create_dir_all(&users_dir).context("Failed to create users directory")?;

    std::fs::create_dir_all(&workflows_dir).context("Failed to create workflows directory")?;

    println!("Created: {}", ukweli_dir.display());
    println!("Created: {}", users_dir.display());
    println!("Created: {}", workflows_dir.display());

    Ok(())
}
