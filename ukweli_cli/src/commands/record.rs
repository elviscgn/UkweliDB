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

    let index = ledger_mgr.append_record(&payload, signers)?;

    println!("\n Record appended successfully!");
    println!("   Index: {}", index);
    println!(
        "   Hash: {}",
        ledger_mgr
            .ledger()
            .records
            .get(index)
            .map(|r| r.record_hash.as_str())
            .unwrap_or("unknown")
    );

    Ok(())
}

pub fn list() -> Result<()> {
    let ledger_mgr = LedgerManager::load()?;

    let records = ledger_mgr.ledger().records.len();

    if records == 0 {
        println!("No records in ledger.");
        return Ok(());
    }

    println!("Records in ledger: {}", records);
    println!();

    for (i, record) in ledger_mgr.ledger().all_records().enumerate() {
        let signer_list = record
            .signers
            .iter()
            .map(|s| s.user_id.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        println!(
            "#{:<4} | {} | Signers: {}",
            i,
            &record.payload.chars().take(50).collect::<String>(),
            signer_list
        );
    }

    Ok(())
}

pub fn show(index: usize) -> Result<()> {
    let ledger_mgr = LedgerManager::load()?;

    let record = ledger_mgr
        .ledger()
        .records
        .get(index)
        .with_context(|| format!("Record #{} not found", index))?;

    println!("Record #{}", record.index);
    println!("─────────────────────────────────────");
    println!("Payload:      {}", record.payload);
    println!("Payload Hash: {}", record.payload_hash);
    println!("Record Hash:  {}", record.record_hash);
    println!("Previous:     {}", record.prev_hash);
    println!("Timestamp:    {}", record.timestamp);
    println!("Nonce:        {}", record.nonce);
    println!("\nSigners:");
    for signer in &record.signers {
        println!(
            "  • {} (roles: {})",
            signer.user_id,
            if signer.roles.is_empty() {
                "none".to_string()
            } else {
                signer
                    .roles
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        );
    }
    println!("\nSignatures:");
    for (user_id, sig) in &record.signatures {
        println!("  • {}: {}", user_id, hex::encode(sig.to_bytes()));
    }

    Ok(())
}
pub fn compact() -> Result<()> {
    let ledger_mgr = LedgerManager::load()?;
    ledger_mgr.compact()?;
    Ok(())
}

pub fn verify() -> Result<()> {
    let ledger_mgr = LedgerManager::load()?;
    ledger_mgr.verify_chain()?;
    Ok(())
}
