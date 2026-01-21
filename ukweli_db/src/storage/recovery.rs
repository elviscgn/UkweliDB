use std::collections::HashMap;
use std::path::Path;

use ed25519_dalek::{Signature, VerifyingKey};

use crate::core::{Ledger, Record, User};
use crate::error::{LedgerError, StorageError};
use crate::storage::append::AppendLog;
use crate::storage::database::DatabaseBody;
use crate::storage::persitence::{SerializableRecord, SerializableUser};
use crate::storage::reader::DatabaseReader;
use crate::storage::writer::DatabaseWriter;

pub struct RecoveryManager;

impl RecoveryManager {
    pub fn recover_ledger<P: AsRef<Path>>(db_path: P) -> Result<Ledger, StorageError> {
        let reader = DatabaseReader::new(&db_path)?;

        match reader.read_and_verify() {
            Ok((_header, body)) => {
                let mut ledger = Self::reconstruct_from_body(body)?;

                if let Ok(mut append_log) = AppendLog::new(&db_path) {
                    match append_log.read_all_entries() {
                        Ok(entries) if !entries.is_empty() => {
                            Self::replay_wal(&mut ledger, entries)?;
                            Self::compact(&db_path, &ledger)?;
                        }
                        _ => {}
                    }
                }

                ledger.verify_chain().map_err(|e| match e {
                    LedgerError::ChainValidation(msg) => StorageError::ValidationFailed(msg),
                    _ => StorageError::ValidationFailed(format!("Ledger error: {:?}", e)),
                })?;

                Ok(ledger)
            }
            Err(StorageError::ChecksumMismatch) => Self::recover_from_wal(&db_path),
            Err(e) => Err(e),
        }
    }

    fn reconstruct_from_body(body: DatabaseBody) -> Result<Ledger, StorageError> {
        let mut ledger = Ledger::new();
        ledger.records.clear();
        ledger.users.clear();
        ledger.verify_registry.clear();

        for ser_user in body.users {
            let verifying_key_bytes: [u8; 32] =
                ser_user.verifying_key_bytes.try_into().map_err(|_| {
                    StorageError::Deserialization("Invalid verifying key length".to_string())
                })?;

            let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes).map_err(|e| {
                StorageError::Deserialization(format!("Invalid verifying key: {}", e))
            })?;

            let mut user = User::new(&ser_user.user_id);
            for role in ser_user.roles {
                user.add_role(&role);
            }

            ledger.users.insert(ser_user.user_id.clone(), user);
            ledger
                .verify_registry
                .insert(ser_user.user_id.clone(), verifying_key);
        }

        for ser_record in body.records {
            let signers: Vec<User> = ser_record
                .signer_ids
                .iter()
                .filter_map(|user_id| ledger.users.get(user_id).cloned())
                .collect();

            if signers.len() != ser_record.signer_ids.len() {
                return Err(StorageError::Deserialization(format!(
                    "Missing signers for record {}",
                    ser_record.index
                )));
            }

            let mut signatures = HashMap::new();
            for (user_id, sig_bytes) in &ser_record.signatures {
                if let Some(sig) = Self::try_parse_signature(sig_bytes) {
                    signatures.insert(user_id.clone(), sig);
                }
            }

            if signatures.len() != ser_record.signatures.len() {
                return Err(StorageError::Deserialization(format!(
                    "Invalid signatures for record {}",
                    ser_record.index
                )));
            }

            let record = Record {
                index: ser_record.index,
                payload: ser_record.payload,
                payload_hash: ser_record.payload_hash,
                signers,
                signatures,
                prev_hash: ser_record.prev_hash,
                record_hash: ser_record.record_hash,
                timestamp: ser_record.timestamp,
                nonce: ser_record.nonce,
            };

            ledger.records.push(record);
        }

        ledger.records.sort_by(|a, b| a.index.cmp(&b.index));

        Ok(ledger)
    }

    fn try_parse_signature(sig_bytes: &[u8]) -> Option<Signature> {
        let arr: [u8; 64] = sig_bytes.try_into().ok()?;
        Some(Signature::from_bytes(&arr))
    }

    fn replay_wal(
        ledger: &mut Ledger,
        entries: Vec<(crate::storage::append::AppendEntry, Vec<u8>)>,
    ) -> Result<(), StorageError> {
        use rkyv::rancor::Error as RkyvError;

        for (entry, data) in entries {
            match entry.entry_type {
                1 => {
                    let archived =
                        rkyv::access::<rkyv::Archived<SerializableRecord>, RkyvError>(&data)
                            .map_err(|e| {
                                StorageError::Deserialization(format!(
                                    "Failed to access WAL record: {}",
                                    e
                                ))
                            })?;

                    let ser_record: SerializableRecord =
                        rkyv::deserialize::<SerializableRecord, RkyvError>(archived).map_err(
                            |e| {
                                StorageError::Deserialization(format!(
                                    "Failed to deserialize WAL record: {}",
                                    e
                                ))
                            },
                        )?;

                    let signers: Vec<User> = ser_record
                        .signer_ids
                        .iter()
                        .filter_map(|user_id| ledger.users.get(user_id).cloned())
                        .collect();

                    if signers.is_empty() {
                        continue;
                    }

                    let mut signatures = HashMap::new();
                    for tuple in &ser_record.signatures {
                        let user_id = &tuple.0;
                        let sig_bytes = tuple.1.as_slice();
                        if let Some(sig) = Self::try_parse_signature(sig_bytes) {
                            signatures.insert(user_id.clone(), sig);
                        }
                    }

                    let record = Record {
                        index: ser_record.index,
                        payload: ser_record.payload,
                        payload_hash: ser_record.payload_hash,
                        signers,
                        signatures,
                        prev_hash: ser_record.prev_hash,
                        record_hash: ser_record.record_hash,
                        timestamp: ser_record.timestamp,
                        nonce: ser_record.nonce,
                    };

                    if !ledger.records.iter().any(|r| r.index == record.index) {
                        ledger.records.push(record);
                    }
                }
                2 => {
                    let archived =
                        rkyv::access::<rkyv::Archived<SerializableUser>, RkyvError>(&data)
                            .map_err(|e| {
                                StorageError::Deserialization(format!(
                                    "Failed to access WAL user: {}",
                                    e
                                ))
                            })?;

                    let ser_user: SerializableUser =
                        rkyv::deserialize::<SerializableUser, RkyvError>(archived).map_err(
                            |e| {
                                StorageError::Deserialization(format!(
                                    "Failed to deserialize WAL user: {}",
                                    e
                                ))
                            },
                        )?;

                    let verifying_key_bytes: [u8; 32] =
                        ser_user.verifying_key_bytes.try_into().map_err(|_| {
                            StorageError::Deserialization(
                                "Invalid verifying key length".to_string(),
                            )
                        })?;

                    let verifying_key =
                        VerifyingKey::from_bytes(&verifying_key_bytes).map_err(|e| {
                            StorageError::Deserialization(format!("Invalid verifying key: {}", e))
                        })?;

                    if !ledger.verify_registry.contains_key(&ser_user.user_id) {
                        let mut user = User::new(&ser_user.user_id);
                        for role in &ser_user.roles {
                            user.add_role(role);
                        }

                        ledger.users.insert(ser_user.user_id.clone(), user);
                        ledger
                            .verify_registry
                            .insert(ser_user.user_id.clone(), verifying_key);
                    }
                }
                _ => {
                    return Err(StorageError::Deserialization(format!(
                        "Unknown entry type: {}",
                        entry.entry_type
                    )));
                }
            }
        }

        Ok(())
    }

    fn recover_from_wal<P: AsRef<Path>>(db_path: P) -> Result<Ledger, StorageError> {
        let mut append_log = AppendLog::new(&db_path)?;
        let entries = append_log.read_all_entries()?;

        if entries.is_empty() {
            return Err(StorageError::ValidationFailed(
                "Cannot recover: both main DB and WAL are corrupted/empty".to_string(),
            ));
        }

        let mut ledger = Ledger::new();
        ledger.records.clear();

        Self::replay_wal(&mut ledger, entries)?;

        ledger.records.sort_by(|a, b| a.index.cmp(&b.index));

        Ok(ledger)
    }

    pub fn compact<P: AsRef<Path>>(db_path: P, ledger: &Ledger) -> Result<(), StorageError> {
        let backup_path = format!("{}.backup", db_path.as_ref().display());
        if db_path.as_ref().exists() {
            std::fs::copy(&db_path, &backup_path)?;
        }

        let mut writer = DatabaseWriter::new(&db_path)?;
        writer.write_ledger(ledger)?;

        if let Ok(mut append_log) = AppendLog::new(&db_path) {
            let _ = append_log.truncate();
        }

        if Path::new(&backup_path).exists() {
            std::fs::remove_file(&backup_path)?;
        }

        Ok(())
    }

    pub fn create_snapshot<P: AsRef<Path>>(
        ledger: &Ledger,
        snapshot_path: P,
    ) -> Result<(), StorageError> {
        let mut writer = DatabaseWriter::new(snapshot_path)?;
        writer.write_ledger(ledger)?;
        Ok(())
    }

    pub fn verify_file<P: AsRef<Path>>(db_path: P) -> Result<bool, StorageError> {
        let reader = DatabaseReader::new(db_path)?;
        reader.read_and_verify()?;
        Ok(true)
    }
}
