use anyhow::Context;
use anyhow::{Result, bail};

use crate::{ledger_manager::LedgerManager, user_store::UserStore};

pub fn append(payload: String, signer_ids: Vec<String>) -> Result<()> {
    if payload.is_empty() {
        bail!("Payload cannot be empty");
    }

    if signer_ids.is_empty() {
        bail!("At least one signer is required");
    }

    println!("Appending record...");
    println!("Payload: {}", payload);
    println!("Signers: {}", signer_ids.join(", "));

    let mut ledger_mgr = LedgerManager::load()?;

    let mut signers = Vec::new();
    for signer_id in &signer_ids {
        let user = UserStore::load_user(signer_id)
            .with_context(|| format!("Failed to load signer '{}'", signer_id))?;

        if !ledger_mgr.ledger().verify_registry.contains_key(signer_id) {
            println!(
                "User '{}' not registered in ledger, attempting to register...",
                signer_id
            );
            ledger_mgr.register_user(user.clone())?;
        }

        signers.push(user);
    }
    Ok(())
}
