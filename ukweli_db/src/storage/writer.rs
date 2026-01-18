use crate::error::StorageError;

use rkyv::rancor::Error as RkyvError;

use std::io::Write;
// use std::io::Write;
use crate::core::Ledger;
use crate::storage::database::{DatabaseBody, DatabaseFooter, DatabaseHeader, HEADER_SIZE};
use crate::storage::persitence::{SerializableRecord, SerializableUser};
use std::fs::{File, OpenOptions};
use std::path::Path;
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
