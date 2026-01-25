use anyhow::{Result, bail};
use std::path::Path;

use crate::config::Config;
use anyhow::Context;
use ukweli_db::{Ledger, core::User};
use ukweli_db::{storage::append::AppendLog, storage::recovery::RecoveryManager};

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

    pub fn register_user(&mut self, user: User) -> Result<()> {
        if self.ledger.verify_registry.contains_key(&user.user_id) {
            bail!(
                "User '{}' is already registered in the ledger",
                user.user_id
            );
        }

        self.ledger.register_user(user.clone());

        let mut append_log = AppendLog::new(&self.db_path).context("Failed to open append log")?;

        append_log
            .append_user(&user)
            .context("Failed to write user to WAL")?;

        println!("User '{}' registered in ledger", user.user_id);

        Ok(())
    }
    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    pub fn append_record(&mut self, payload: &str, signers: Vec<User>) -> Result<usize> {
        let index = self
            .ledger
            .add_record(payload, signers.clone())
            .context("Failed to add record to ledger")?;

        let record = self
            .ledger
            .records
            .get(index)
            .context("Record was added but cannot be retrieved")?;

        let mut append_log = AppendLog::new(&self.db_path).context("Failed to open append log")?;

        append_log
            .append_record(record)
            .context("Failed to write record to WAL")?;

        println!("Record #{} appended to WAL", index);

        Ok(index)
    }

    pub fn compact(&self) -> Result<()> {
        // from wal to main db
        // TODO: automate this
        println!("Compacting database...");
        RecoveryManager::compact(&self.db_path, &self.ledger)
            .context("Failed to compact database")?;

        println!("Database compacted");

        Ok(())
    }
}
