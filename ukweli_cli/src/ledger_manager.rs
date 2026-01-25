use anyhow::{Result, bail};
use std::path::Path;

use crate::config::Config;
use anyhow::Context;
use ukweli_db::{Ledger, storage::recovery::RecoveryManager};

pub struct LedgerManager {
    pub ledger: Ledger,
    db_path: std::path::PathBuf,
}

impl LedgerManager {
    pub fn load() -> Result<Self> {
        let config = Config::load_or_default()?;
        Self::load_from_path(&config.db_path)
    }

    pub fn load_from_path<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref();

        if !db_path.exists() {
            bail!(
                "Database not found at: {}\nRun 'ukweli init' first.",
                db_path.display()
            );
        }

        println!("Loading ledger from: {}", db_path.display());

        let ledger = RecoveryManager::recover_ledger(db_path).context("Failed to load ledger")?;

        println!("Loaded {} records", ledger.length());
        
        Ok(Self {
            ledger,
            db_path: db_path.to_path_buf(),
        })
    }
}
