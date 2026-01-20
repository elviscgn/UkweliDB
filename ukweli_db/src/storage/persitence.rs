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
    #![allow(clippy::indexing_slicing)]

    use super::*;

    use crate::core::{Ledger, User};
    use crate::storage::database::{DatabaseHeader, MAGIC_NUMBER};
    use crate::storage::reader::DatabaseReader;
    use crate::storage::writer::DatabaseWriter;

    use std::fs;

    // =========
    // header tests
    // =========
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
    fn test_header_timestamps() {
        let header1 = DatabaseHeader::new(10, 128, 1000);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let header2 = DatabaseHeader::new(10, 128, 1000);

        // timestamps should be diff
        assert!(header2.created_timestamp >= header1.created_timestamp);
        assert!(header2.last_modified >= header1.last_modified);
    }

    #[test]
    fn test_header_serialization() {
        let header = DatabaseHeader::new(50, 128, 2500);

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&header).unwrap();

        let archived =
            rkyv::access::<rkyv::Archived<DatabaseHeader>, rkyv::rancor::Error>(&bytes).unwrap();

        let deserialized =
            rkyv::deserialize::<DatabaseHeader, rkyv::rancor::Error>(archived).unwrap();

        // verify
        assert_eq!(deserialized.magic, header.magic);
        assert_eq!(deserialized.version_major, header.version_major);
        assert_eq!(deserialized.version_minor, header.version_minor);
        assert_eq!(deserialized.record_count, header.record_count);
        assert_eq!(deserialized.body_offset, header.body_offset);
        assert_eq!(deserialized.footer_offset, header.footer_offset);
    }

    // =========
    // record tests
    // =========
    #[test]
    fn test_serializable_record_conversion() {
        let mut ledger = Ledger::new();
        let user = User::new("test_user");
        ledger.register_user(user.clone());
        ledger.add_record("test payload", vec![user]).unwrap();

        let record = &ledger.records[1]; // Skip genesis
        let serializable = SerializableRecord::from(record);

        assert_eq!(serializable.index, record.index);
        assert_eq!(serializable.payload, record.payload);
        assert_eq!(serializable.payload_hash, record.payload_hash);
        assert_eq!(serializable.record_hash, record.record_hash);
        assert_eq!(serializable.signer_ids.len(), 1);
        assert_eq!(serializable.signatures.len(), 1);
    }

    #[test]
    fn test_serializable_record_multi_signer() {
        let mut ledger = Ledger::new();
        let user1 = User::new("signer1");
        let user2 = User::new("signer2");
        let user3 = User::new("signer3");

        ledger.register_user(user1.clone());
        ledger.register_user(user2.clone());
        ledger.register_user(user3.clone());

        ledger
            .add_record("multi-sig", vec![user1, user2, user3])
            .unwrap();

        let record = &ledger.records[1];
        let serializable = SerializableRecord::from(record);

        assert_eq!(serializable.signer_ids.len(), 3);
        assert_eq!(serializable.signatures.len(), 3);
    }

    #[test]
    fn test_serializable_user_conversion() {
        let mut user = User::new("test_user");
        user.add_role("admin");
        user.add_role("editor");

        let serializable = SerializableUser::from(&user);

        assert_eq!(serializable.user_id, "test_user");
        assert_eq!(serializable.verifying_key_bytes.len(), 32);
        assert_eq!(serializable.roles.len(), 2);
        assert!(serializable.roles.contains(&"admin".to_string()));
        assert!(serializable.roles.contains(&"editor".to_string()));
    }

    #[test]
    fn test_serializable_user_no_roles() {
        let user = User::new("no_roles");
        let serializable = SerializableUser::from(&user);

        assert_eq!(serializable.user_id, "no_roles");
        assert_eq!(serializable.roles.len(), 0);
    }

    #[test]
    fn test_writer_creates_file() {
        let test_path = "test_writer_creates.db";
        let _ = fs::remove_file(test_path);

        assert!(!std::path::Path::new(test_path).exists());

        let _writer = DatabaseWriter::new(test_path).unwrap();
        assert!(std::path::Path::new(test_path).exists());

        fs::remove_file(test_path).unwrap();
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
        // fs::remove_file(test_path).unwrap();
    }
}
