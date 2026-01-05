use std::{collections::HashMap, fmt::format};

use crate::{LedgerError, core::User};
use ed25519_dalek::{VerifyingKey, ed25519::signature};
use sha256::digest;

use super::record::Record;

pub const GENESIS_PREV_HASH: &str = "00000000";

#[derive(Debug)]
pub struct Ledger {
    pub records: Vec<Record>,
    pub users: HashMap<String, User>,
    pub verify_registry: HashMap<String, VerifyingKey>, // (userid, vkey)
}

impl From<ed25519_dalek::SignatureError> for LedgerError {
    fn from(err: ed25519_dalek::SignatureError) -> Self {
        LedgerError::ChainValidation(err.to_string())
    }
}

impl Ledger {
    pub fn new() -> Self {
        // todo genesis more complex in the future
        let genesis_user = User::new("GENESIS");
        let mut genesis_record =
            Record::new(0, "Genesis", GENESIS_PREV_HASH, vec![genesis_user.clone()]);

        let user_id = genesis_user.user_id.clone();
        let verifying_key = genesis_user.verifying_key;

        let mut users = HashMap::new();
        let mut verify_registry = HashMap::new();

        users.insert(user_id.clone(), genesis_user.clone());
        verify_registry.insert(user_id, verifying_key);

        let signature = genesis_user.sign(genesis_record.record_hash.as_bytes());
        genesis_record
            .signatures
            .insert(genesis_user.user_id, signature);

        Self {
            records: vec![genesis_record],
            users,
            verify_registry,
        }
    }

    pub fn add_record(&mut self, payload: &str, signers: Vec<User>) -> Result<usize, LedgerError> {
        for signer in &signers {
            if !self.verify_registry.contains_key(&signer.user_id) {
                return Err(LedgerError::UnregistedUser);
            }
        }
        let last_record = match self.get_last_record() {
            Some(record) => record,
            None => return Err(LedgerError::RecordAccessFailed),
        };

        if payload.is_empty() {
            return Err(LedgerError::EmptyPayload);
        }
        let record = Record::new(
            last_record.index + 1,
            payload,
            &last_record.record_hash,
            signers,
        );
        let ret_index = record.index;
        self.records.push(record);

        Ok(ret_index)
    }

    fn get_last_record(&self) -> Option<&Record> {
        self.records.last()
    }

    pub fn register_user(&mut self, user: User) {
        let user_id = user.user_id.clone();
        let verifying_key = user.verifying_key;

        self.users.insert(user_id.clone(), user);
        self.verify_registry.insert(user_id, verifying_key);
    }

    pub fn length(&self) -> usize {
        self.records.len()
    }

    pub fn all_records(&self) -> impl Iterator<Item = &Record> {
        self.records.iter()
    }

    pub fn all_users(&self) -> impl Iterator<Item = (&String, &User)> {
        self.users.iter()
    }

    fn verify_signatures(&self, record: &Record) -> Result<bool, LedgerError> {
        for signer in &record.signers {
            let verify_key: Result<&VerifyingKey, LedgerError> = self
                .verify_registry
                .get(&signer.user_id)
                .ok_or(LedgerError::ChainValidation(format!(
                    "Unknown signer {:?}",
                    signer.user_id
                )));

            let signature =
                record
                    .signatures
                    .get(&signer.user_id)
                    .ok_or(LedgerError::ChainValidation(format!(
                        "Missing signature from {}",
                        signer.user_id
                    )));

            verify_key?.verify_strict(record.record_hash.as_bytes(), signature?)?;
        }
        Ok(true)
    }

    pub fn verify_chain(&self) -> Result<bool, LedgerError> {
        for (i, record) in self.records.iter().enumerate() {
            if i == 0 {
                if record.prev_hash != GENESIS_PREV_HASH {
                    return Err(LedgerError::ChainValidation("Invalid genesis".to_string()));
                }
            } else {
                let prev_record = self
                    .records
                    .get(i - 1)
                    .ok_or(LedgerError::RecordAccessFailed)?;
                if record.prev_hash != prev_record.record_hash {
                    return Err(LedgerError::ChainValidation(format!(
                        "Broken chain at {}",
                        i,
                    )));
                }
            }

            let computed_payload_hash = digest(&record.payload);
            if computed_payload_hash != record.payload_hash {
                return Err(LedgerError::ChainValidation(format!(
                    "Payload tampered at {}",
                    i,
                )));
            }

            let joined_signers = record
                .signers
                .iter()
                .map(|u| u.user_id.clone())
                .collect::<Vec<String>>()
                .join(",");

            let material = format!(
                "{} {} {} {}",
                record.index, record.prev_hash, record.payload_hash, joined_signers
            );
            let computed_record_hash = digest(material);
            if computed_record_hash != record.record_hash {
                return Err(LedgerError::ChainValidation(format!(
                    "Record hash mismatch at {}",
                    i,
                )));
            }

            self.verify_signatures(record).map_err(|e| {
                LedgerError::ChainValidation(format!("Signature validation failed: {}", e))
            })?;
        }
        Ok(true)
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    // only in tests :) I want them to panic here but never during runtime
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    #![allow(clippy::indexing_slicing)]
    #![allow(clippy::panic)]
    #![allow(clippy::unreachable)]
    #![allow(clippy::assertions_on_result_states)]

    use super::*;

    #[test]
    fn test_ledger_init() {
        let db = Ledger::new();

        assert_eq!(db.records[0].index, 0);
        assert_eq!(db.length(), 1);
        assert!(db.users.contains_key("GENESIS"));
        assert!(db.verify_registry.contains_key("GENESIS"));
    }

    #[test]
    fn test_add_record() {
        let mut ledger = Ledger::new();

        let test_signer = User::new("user1");
        ledger.register_user(test_signer.clone());
        let result = ledger.add_record("test payload", vec![test_signer]);
        assert!(result.is_ok());
        assert_eq!(ledger.length(), 2);

        let added_record = &ledger.records[1];
        assert_eq!(added_record.payload, "test payload");
        assert_eq!(added_record.index, 1);

        // adding record with unregistered user
        let unreg_signer = User::new("unreg_user");
        let result = ledger.add_record("another payload", vec![unreg_signer]);
        assert!(result.is_err());

        // adding record with empty payload
        let reg_signer = User::new("reg_user");
        ledger.register_user(reg_signer.clone());
        let result = ledger.add_record("", vec![reg_signer]);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_chain_valid() {
        let mut ledger = Ledger::new();

        let test_signer1 = User::new("user1");
        let test_signer2 = User::new("user2");
        let test_signer3 = User::new("user3");

        ledger.register_user(test_signer1.clone());
        ledger.register_user(test_signer2.clone());
        ledger.register_user(test_signer3.clone());

        ledger
            .add_record("pay 100", vec![test_signer1, test_signer2])
            .unwrap();
        ledger.add_record("sell 50", vec![test_signer3]).unwrap();

        assert!(ledger.verify_chain().unwrap());
    }

    #[test]
    fn test_verify_chain_tampered() {
        let mut ledger = Ledger::new();

        let test_signer1 = User::new("user1");
        let test_signer2 = User::new("user2");
        let test_signer3 = User::new("user3");

        ledger.register_user(test_signer1.clone());
        ledger.register_user(test_signer2.clone());
        ledger.register_user(test_signer3.clone());

        ledger
            .add_record("pay 100", vec![test_signer1, test_signer2])
            .unwrap();
        ledger.add_record("sell 50", vec![test_signer3]).unwrap();

        // Tamper with data
        ledger.records[1].payload = "evil data".to_string();

        // Tampered chain should fail verification
        let result = ledger.verify_chain();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_handling() {
        let mut ledger = Ledger::new();
        let unreg_signer = User::new("unreg_user");
        let result = ledger.add_record("test payload", vec![unreg_signer]);
        assert!(matches!(result, Err(LedgerError::UnregistedUser)));

        let empty_payload_signer = User::new("reg_user");
        ledger.register_user(empty_payload_signer.clone());
        let result = ledger.add_record("", vec![empty_payload_signer]);
        assert!(matches!(result, Err(LedgerError::EmptyPayload)));
    }

    #[test]
    fn test_hash_calculation() {
        let mut ledger = Ledger::new();
        let test_signer1 = User::new("user1");
        ledger.register_user(test_signer1.clone());

        let record1_hash = ledger.records[0].record_hash.clone();
        ledger.add_record("test", vec![test_signer1]).unwrap();

        let record2_hash = ledger.records[1].record_hash.clone();

        // Hashes should be different
        assert_ne!(record1_hash, record2_hash);

        assert_eq!(record1_hash.len(), 64);
        assert_eq!(record2_hash.len(), 64);
    }

    #[test]
    fn test_comprehensive_scenario() {
        let mut ledger = Ledger::new();

        let user1 = User::new("Elvis");
        let user2 = User::new("Thabo");
        let user3 = User::new("Kamau");
        let user4 = User::new("Kipchoge");
        let user5 = User::new("Amina");
        let user6 = User::new("Zuri");

        ledger.register_user(user1.clone());
        ledger.register_user(user2.clone());
        ledger.register_user(user3.clone());
        ledger.register_user(user4.clone());
        ledger.register_user(user5.clone());
        ledger.register_user(user6.clone());

        let transactions = [
            "Elvis pays Thabo 100",
            "Kamau pays Kipchoge 50",
            "Amina pays Zuri 200",
        ];

        ledger
            .add_record(transactions[0], vec![user1, user2])
            .unwrap();
        ledger
            .add_record(transactions[1], vec![user3, user4])
            .unwrap();
        ledger
            .add_record(transactions[2], vec![user5, user6])
            .unwrap();

        assert!(ledger.verify_chain().unwrap());

        assert_eq!(ledger.length(), 4);

        ledger.records[2].payload = "HACKED!".to_string();
        let result = ledger.verify_chain();
        assert!(result.is_err());
    }
}
