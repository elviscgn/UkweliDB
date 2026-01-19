use crate::core::User;
use rkyv::bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

// use std::io::Write;
use crate::core::Record;

#[derive(Archive, Serialize, Deserialize, Debug, Clone, CheckBytes)]
#[rkyv(derive(Debug))]
pub struct SerializableRecord {
    pub index: usize,
    pub payload: String,
    pub payload_hash: String,
    pub signer_ids: Vec<String>,
    pub signatures: Vec<(String, Vec<u8>)>, // (user_id, signature_bytes)
    pub prev_hash: String,
    pub record_hash: String,
    pub timestamp: u64,
    pub nonce: u64,
}

impl From<&Record> for SerializableRecord {
    fn from(record: &Record) -> Self {
        Self {
            index: record.index,
            payload: record.payload.clone(),
            payload_hash: record.payload_hash.clone(),

            signer_ids: record.signers.iter().map(|u| u.user_id.clone()).collect(),

            signatures: record
                .signatures
                .iter()
                .map(|(id, sig)| (id.clone(), sig.to_bytes().to_vec()))
                .collect(),

            prev_hash: record.prev_hash.clone(),
            record_hash: record.record_hash.clone(),
            timestamp: record.timestamp,
            nonce: record.nonce,
        }
    }
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, CheckBytes)]
pub struct SerializableUser {
    pub user_id: String,
    pub verifying_key_bytes: Vec<u8>,
    pub roles: Vec<String>,
}

impl From<&User> for SerializableUser {
    fn from(user: &User) -> Self {
        Self {
            user_id: user.user_id.clone(),
            verifying_key_bytes: user.verifying_key.to_bytes().to_vec(),
            roles: user.roles.iter().cloned().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    // use super::*;

    use crate::core::{Ledger, User};
    use crate::storage::database::{DatabaseHeader, MAGIC_NUMBER};
    use crate::storage::reader::DatabaseReader;
    use crate::storage::writer::DatabaseWriter;
    use std::fs;

    #[test]
    fn test_header_creation() {
        let header = DatabaseHeader::new(100, 128, 5000);

        assert_eq!(header.magic, MAGIC_NUMBER);
        assert_eq!(header.version_major, 1);
        assert_eq!(header.version_minor, 0);
        assert_eq!(header.record_count, 100);
        assert_eq!(header.body_offset, 128);
        assert_eq!(header.footer_offset, 5000);
        assert_eq!(header.reserved.len(), 40);

        // all reserved bytes should be zero
        assert!(header.reserved.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_write_and_read_ledger() {
        let mut ledger = Ledger::new();

        let user1 = User::new("0xElvis");
        let user2 = User::new("0xChege");

        ledger.register_user(user1.clone());
        ledger.register_user(user2.clone());

        ledger
            .add_record("First transaction", vec![user1.clone()])
            .unwrap();
        ledger
            .add_record("Second transaction", vec![user1, user2])
            .unwrap();

        let test_path = "test_db.ukweli";

        // Write
        let mut writer = DatabaseWriter::new(test_path).unwrap();
        writer.write_ledger(&ledger).unwrap();

        // read
        let reader = DatabaseReader::new(test_path).unwrap();
        let (header, body) = reader.read_and_verify().unwrap();

        assert_eq!(header.magic, MAGIC_NUMBER);
        assert_eq!(header.version_major, 1);
        assert_eq!(header.version_minor, 0);
        assert_eq!(header.record_count, 3); // Genesis + 2 records
        assert_eq!(body.records.len(), 3);

        // cleanup
        fs::remove_file(test_path).unwrap();
    }
}
