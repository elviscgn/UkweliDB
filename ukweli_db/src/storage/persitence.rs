use crate::core::User;
use crate::error::StorageError;
use rkyv::bytecheck::CheckBytes;
use rkyv::rancor::Error as RkyvError; // The standard error type for rkyv 0.8
use rkyv::{Archive, Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
// use std::io::Write;
use crate::core::{Ledger, Record};
use std::path::Path;

pub const MAGIC_NUMBER: [u8; 4] = [0x55, 0x4B, 0x57, 0x4C]; // "UKWL"
pub const VERSION_MAJOR: u8 = 1;
pub const VERSION_MINOR: u8 = 0; // might be redundant but good to keep for now
pub const HEADER_SIZE: usize = 64;

// TODO
// https://github.com/elviscgn/UkweliDB/issues/1#issuecomment-3734544932
#[derive(Archive, Serialize, Deserialize, Debug, CheckBytes)]
#[rkyv(derive(Debug))]
#[repr(C)]
pub struct DatabaseHeader {
    // identity stuff (6 bytes)
    pub magic: [u8; 4],
    pub version_major: u8,
    pub version_minor: u8,

    // flags (2 bytes)
    // pub flags: u16, | for stuff like compression and encryption TBD
    pub record_count: u64, // 8b

    pub created_timestamp: u64, // 8 each
    pub last_modified: u64,

    pub body_offset: u64, // where the body and footer start
    pub footer_offset: u64,

    pub checksum: [u8; 32], // hash of body content
    pub reserved: [u8; 40],
} // Total: 6 + 8 + 16 + 16 + 32 + 40 = 118 bytes

impl DatabaseHeader {
    pub fn new(record_count: u64, body_offset: u64, footer_offset: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            magic: MAGIC_NUMBER,
            version_major: VERSION_MAJOR,
            version_minor: VERSION_MINOR,
            record_count,
            created_timestamp: now,
            last_modified: now,
            body_offset,
            footer_offset,
            checksum: [0; 32],
            reserved: [0; 40],
        }
    }
}

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

#[derive(Archive, Serialize, Deserialize, Debug, CheckBytes)]

pub struct DatabaseBody {
    pub records: Vec<SerializableRecord>,
    pub users: Vec<SerializableUser>,
}

#[derive(Archive, Serialize, Deserialize, Debug, CheckBytes)]
pub struct DatabaseFooter {
    pub integrity_hash: [u8; 32], // sha256 of entire file before footer
    pub total_file_size: u64,
}

pub struct DatabaseWriter {
    file: File,
}

impl DatabaseWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        Ok(Self { file })
    }

    pub fn write_ledger(&mut self, ledger: &Ledger) -> Result<(), StorageError> {
        let records: Vec<SerializableRecord> = ledger
            .records
            .iter()
            .map(SerializableRecord::from)
            .collect();

        let users: Vec<SerializableUser> =
            ledger.users.values().map(SerializableUser::from).collect();

        let body = DatabaseBody { records, users };

        let body_bytes = rkyv::to_bytes::<RkyvError>(&body)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        let body_checksum = sha256::digest(body_bytes.as_slice());
        let checksum_bytes: [u8; 32] = body_checksum
            .as_bytes()
            .try_into()
            .map_err(|_| StorageError::Serialization("Checksum conversion failed".to_string()))?;

        let body_offset = HEADER_SIZE as u64;
        let footer_offset = body_offset + body_bytes.len() as u64;

        let mut header =
            DatabaseHeader::new(ledger.records.len() as u64, body_offset, footer_offset);
        header.checksum = checksum_bytes;

        let header_bytes = rkyv::to_bytes::<RkyvError>(&header)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        let mut pre_footer_data = Vec::with_capacity(header_bytes.len() + body_bytes.len());
        pre_footer_data.extend_from_slice(&header_bytes);
        pre_footer_data.extend_from_slice(&body_bytes);

        let integrity_hash = sha256::digest(&pre_footer_data);
        let integrity_bytes: [u8; 32] = integrity_hash.as_bytes().try_into().map_err(|_| {
            StorageError::Serialization("Integrity hash conversion failed".to_string())
        })?;

        let footer = DatabaseFooter {
            integrity_hash: integrity_bytes,
            total_file_size: (HEADER_SIZE + body_bytes.len() + 64) as u64,
        };

        let footer_bytes = rkyv::to_bytes::<RkyvError>(&footer)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        self.file.write_all(&header_bytes)?;

        let padding_needed = HEADER_SIZE.saturating_sub(header_bytes.len());
        if padding_needed > 0 {
            let padding = vec![0u8; padding_needed];
            self.file.write_all(&padding)?;
        }

        self.file.write_all(&body_bytes)?;
        self.file.write_all(&footer_bytes)?;
        self.file.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    // use crate::core::Ledger;
    // use std::fs;

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
}
