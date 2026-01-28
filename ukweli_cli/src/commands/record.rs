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

pub fn list(
    signer: Option<String>,
    from: Option<usize>,
    to: Option<usize>,
    limit: Option<usize>,
) -> Result<()> {
    let ledger_mgr = LedgerManager::load()?;

    let all_records: Vec<_> = ledger_mgr.ledger().all_records().collect();

    if all_records.is_empty() {
        println!("No records in ledger.");
        return Ok(());
    }

    let mut filtered_records = Vec::new();

    for record in all_records {
        if let Some(from_idx) = from {
            if record.index < from_idx {
                continue;
            }
        }

        if let Some(to_idx) = to {
            if record.index > to_idx {
                continue;
            }
        }

        if let Some(ref signer_id) = signer {
            let has_signer = record.signers.iter().any(|s| s.user_id == *signer_id);

            if !has_signer {
                continue;
            }
        }

        filtered_records.push(record);
    }

    if let Some(lim) = limit {
        filtered_records.truncate(lim);
    }

    if filtered_records.is_empty() {
        println!("No records match the filters.");
        return Ok(());
    }

    println!("Found {} record(s)", filtered_records.len());

    let mut active_filters = Vec::new();
    if let Some(s) = &signer {
        active_filters.push(format!("Signer: {}", s));
    }
    if let Some(f) = from {
        active_filters.push(format!("From index: {}", f));
    }
    if let Some(t) = to {
        active_filters.push(format!("To index: {}", t));
    }
    if let Some(l) = limit {
        active_filters.push(format!("Limit: {}", l));
    }

    if !active_filters.is_empty() {
        println!("\nFilters:");
        for filter in active_filters {
            println!("  • {}", filter);
        }
    }

    println!();

    for record in filtered_records {
        let signer_list = record
            .signers
            .iter()
            .map(|s| s.user_id.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        let display_payload = if record.payload.len() > 60 {
            format!(
                "{}...",
                &record.payload.chars().take(57).collect::<String>()
            )
        } else {
            record.payload.clone()
        };

        println!(
            "#{:<4} | {} | Signers: {}",
            record.index, display_payload, signer_list
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
