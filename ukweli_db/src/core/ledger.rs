use std::collections::HashMap;

use crate::core::User;
use ed25519_dalek::VerifyingKey;
use sha256::digest;

use super::record::Record;

#[derive(Debug)]
pub struct Ledger {
    pub records: Vec<Record>,
    pub users: HashMap<String, User>,
    pub verify_registry: HashMap<String, VerifyingKey>,
}

impl Ledger {
    pub fn new() -> Self {
        // todo genesis more complex in the future
        let genesis_user = User::new("GENESIS");
        let genesis_record = Record::new(0, "Genesis", "00000000");

        let user_id = genesis_user.user_id.clone();
        let verifying_key = genesis_user.verifying_key;

        let mut users = HashMap::new();
        let mut verify_registry = HashMap::new();

        users.insert(user_id.clone(), genesis_user);
        verify_registry.insert(user_id, verifying_key);

        Self {
            records: vec![genesis_record],
            users,
            verify_registry,
        }
    }

    pub fn add_record(&mut self, payload: &str) -> Result<usize, String> {
        let last_record = match self.get_last_record() {
            Some(record) => record,
            None => return Err("System error: Could not access previous record.".to_string()),
        };

        if payload.is_empty() {
            return Err("Cannot add an empty payload".to_string());
        }
        let record = Record::new(last_record.index + 1, payload, &last_record.record_hash);
        let ret_index = record.index;
        self.records.push(record);

        Ok(ret_index)
    }

    fn get_last_record(&self) -> Option<&Record> {
        self.records.last()
    }

    pub fn register_user() {}

    pub fn length(&self) -> usize {
        self.records.len()
    }

    pub fn all_records(&self) -> impl Iterator<Item = &Record> {
        self.records.iter()
    }

    pub fn all_users(&self) -> impl Iterator<Item = (&String, &User)> {
        self.users.iter()
    }

    pub fn verify_chain(&self) -> bool {
        // for record in self.records.iter() {
        //     let computed_hash = digest(record.payload.to_owned());
        //     if computed_hash != record.hash {
        //         return false;
        //     }
        // }
        true
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)]
    #[allow(clippy::expect_used)]
    #[allow(clippy::indexing_slicing)] // only in tests :) I want them to panic here but never during runtime
    fn test_ledger_init() {
        let db = Ledger::new();

        assert_eq!(db.records[0].index, 0);
        assert_eq!(db.length(), 1);
        assert!(db.users.contains_key("GENESIS"));
        assert!(db.verify_registry.contains_key("GENESIS"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    #[allow(clippy::expect_used)]
    #[allow(clippy::indexing_slicing)]
    fn test_records() {
        let mut db = Ledger::new();

        assert_eq!(db.records[0].index, 0);
        assert_eq!(db.length(), 1);

        let last_record = db.get_last_record().cloned().unwrap();
        let appended_i = db.add_record("pay 100").expect("Failed to add record");

        println!("--- Records ----");
        for (index, record) in db.all_records().enumerate() {
            let display_hash: String = record.record_hash.chars().take(8).collect();
            let display_prevhash: String = record.prev_hash.chars().take(8).collect();
            println!(
                "{}: [Payload: {}] [Hash: {}...] [Prev-Hash: {}...]",
                index, record.payload, display_hash, display_prevhash
            );
        }
        println!("-----------------");

        println!("--- Users ----");
        for (index, (user_id, user)) in db.all_users().enumerate() {
            // let display_verify_key: String = user.verifying_key.chars().take(8).collect();            // let display_prevhash: String = record.prev_hash.chars().take(8).collect();
            println!(
                "{}: [User: {}] [Verifying Key: {:#?}...]",
                index, user_id, user.verifying_key
            );
        }
        println!("-----------------");
        assert_eq!(appended_i, 1);

        // test prev hash = has of prev
        assert_eq!(db.records[appended_i].prev_hash, last_record.record_hash);

        db.add_record("sell 100").expect("Failed to add record");

        // modify data
        db.records[1].payload = "evil data bahaha".to_owned();

        println!("--- Records ----");
        for (index, record) in db.all_records().enumerate() {
            let display_hash: String = record.record_hash.chars().take(8).collect();
            let display_prevhash: String = record.prev_hash.chars().take(8).collect();
            println!(
                "{}: [Payload: {}] [Hash: {}...] [Prev-Hash: {}...]",
                index, record.payload, display_hash, display_prevhash
            );
        }
        println!("-----------------");

        assert!(!db.verify_chain());
    }

    #[test]
    fn it_works() {
        let input = String::from("hello");
        let val = digest(input);
        assert_eq!(
            val,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
