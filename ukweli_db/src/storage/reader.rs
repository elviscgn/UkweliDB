use crate::error::StorageError;
use crate::storage::database::{DatabaseBody, DatabaseHeader, HEADER_SIZE, MAGIC_NUMBER};
use rkyv::rancor::Error as RkyvError;
use std::fs;
use std::path::Path;

pub struct DatabaseReader {
    buffer: Vec<u8>,
}

impl DatabaseReader {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        // snapshot the file into memory to avoid Undefined Behavior if disk state changes during read.
        let buffer = fs::read(path)?;
        Ok(Self { buffer })
    }

    pub fn read_and_verify(&self) -> Result<(DatabaseHeader, DatabaseBody), StorageError> {
        let header_slice = self.buffer.get(..HEADER_SIZE).ok_or_else(|| {
            StorageError::Serialization("File truncated: missing header".to_string())
        })?;

        let archived_header =
            rkyv::access::<rkyv::Archived<DatabaseHeader>, RkyvError>(header_slice)
                .map_err(|e| StorageError::Deserialization(format!("Header validation: {}", e)))?;

        if archived_header.magic != MAGIC_NUMBER {
            return Err(StorageError::InvalidMagic);
        }

        if archived_header.version_major != 1 {
            return Err(StorageError::UnsupportedVersion(
                archived_header.version_major,
                archived_header.version_minor,
            ));
        }

        let header: DatabaseHeader =
            rkyv::deserialize::<DatabaseHeader, RkyvError>(archived_header)
                .map_err(|e| StorageError::Deserialization(format!("Header map error: {}", e)))?;

        let body_start = header.body_offset as usize;
        let body_end = header.footer_offset as usize;

        let body_bytes = self.buffer.get(body_start..body_end).ok_or_else(|| {
            StorageError::Serialization("Header offsets point outside file boundaries".to_string())
        })?;

        let computed_checksum = sha256::digest(body_bytes);
        let computed_bytes: [u8; 32] = hex::decode(&computed_checksum)
            .map_err(|_| StorageError::Deserialization("Hash conversion error".to_string()))?
            .try_into()
            .map_err(|_| StorageError::Deserialization("Hash conversion error".to_string()))?;

        if computed_bytes != header.checksum {
            return Err(StorageError::ChecksumMismatch);
        }

        let archived_body = rkyv::access::<rkyv::Archived<DatabaseBody>, RkyvError>(body_bytes)
            .map_err(|e| StorageError::Deserialization(format!("Body corruption: {}", e)))?;

        let body: DatabaseBody = rkyv::deserialize::<DatabaseBody, RkyvError>(archived_body)
            .map_err(|e| StorageError::Deserialization(format!("Body map error: {}", e)))?;

        Ok((header, body))
    }
}
