use anyhow::Result;
use std::path::PathBuf;

pub fn run(db_path: Option<PathBuf>) -> Result<()> {
    println!("Initializing Ukweli database...\n");
    println!("Path {:?}", db_path);

    Ok(())
}
