// FILE LOCATION: src/storage/append.rs
// Handles incremental append operations for efficiency (Write-Ahead Log)

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::core::{Record, User};
use crate::error::StorageError;
use crate::storage::persitence::{SerializableRecord, SerializableUser};

const APPEND_MAGIC: [u8; 4] = [0x41, 0x50, 0x4E, 0x44]; // "APND"
const ENTRY_HEADER_SIZE: usize = 4 + 1 + 8 + 4 + 32; // 49 bytes total, no padding needed

#[derive(Debug, Clone)]
pub struct AppendEntry {
    pub magic: [u8; 4],
    pub entry_type: u8, // 1 = Record, 2 = User
    pub timestamp: u64,
    pub data_size: u32,
    pub checksum: [u8; 32],
}

impl AppendEntry {
    pub fn new(entry_type: u8, data_size: u32, checksum: [u8; 32]) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            magic: APPEND_MAGIC,
            entry_type,
            timestamp: now,
            data_size,
            checksum,
        }
    }

    pub fn to_bytes(&self) -> [u8; ENTRY_HEADER_SIZE] {
        let mut bytes = [0u8; ENTRY_HEADER_SIZE];

        // Write magic (4 bytes)
        bytes[0..4].copy_from_slice(&self.magic);

        // Write entry_type (1 byte)
        bytes[4] = self.entry_type;

        // Write timestamp (8 bytes)
        bytes[5..13].copy_from_slice(&self.timestamp.to_le_bytes());

        // Write data_size (4 bytes)
        bytes[13..17].copy_from_slice(&self.data_size.to_le_bytes());

        // Write checksum (32 bytes)
        bytes[17..49].copy_from_slice(&self.checksum);

        bytes
    }

    pub fn from_bytes(bytes: &[u8; ENTRY_HEADER_SIZE]) -> Result<Self, StorageError> {
        // Use get() to avoid indexing/slicing panic
        let magic_slice = bytes.get(0..4).ok_or_else(|| {
            StorageError::Deserialization("Failed to read magic bytes".to_string())
        })?;

        let magic: [u8; 4] = magic_slice.try_into().map_err(|_| {
            StorageError::Deserialization("Failed to convert magic bytes".to_string())
        })?;

        let entry_type = *bytes.get(4).ok_or_else(|| {
            StorageError::Deserialization("Failed to read entry_type".to_string())
        })?;

        let timestamp_slice = bytes.get(5..13).ok_or_else(|| {
            StorageError::Deserialization("Failed to read timestamp bytes".to_string())
        })?;

        let timestamp = u64::from_le_bytes(timestamp_slice.try_into().map_err(|_| {
            StorageError::Deserialization("Failed to convert timestamp bytes".to_string())
        })?);

        let data_size_slice = bytes.get(13..17).ok_or_else(|| {
            StorageError::Deserialization("Failed to read data_size bytes".to_string())
        })?;

        let data_size = u32::from_le_bytes(data_size_slice.try_into().map_err(|_| {
            StorageError::Deserialization("Failed to convert data_size bytes".to_string())
        })?);

        let checksum_slice = bytes.get(17..49).ok_or_else(|| {
            StorageError::Deserialization("Failed to read checksum bytes".to_string())
        })?;

        let checksum: [u8; 32] = checksum_slice.try_into().map_err(|_| {
            StorageError::Deserialization("Failed to convert checksum bytes".to_string())
        })?;

        Ok(Self {
            magic,
            entry_type,
            timestamp,
            data_size,
            checksum,
        })
    }
}

pub struct AppendLog {
    path: PathBuf,
    file: File,
}

impl AppendLog {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, StorageError> {
        let mut append_path = PathBuf::from(db_path.as_ref());
        append_path.set_extension("wal"); // Write-Ahead Log

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&append_path)?;

        Ok(Self {
            path: append_path,
            file,
        })
    }

    pub fn append_record(&mut self, record: &Record) -> Result<(), StorageError> {
        let serializable = SerializableRecord::from(record);

        // Serialize record data using to_bytes
        let data_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&serializable)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        // Calculate checksum with hex decode
        let checksum_str = sha256::digest(data_bytes.as_slice());
        let checksum: [u8; 32] = hex::decode(&checksum_str)
            .map_err(|e| StorageError::Serialization(format!("Hex decode failed: {}", e)))?
            .try_into()
            .map_err(|_| StorageError::Serialization("Checksum conversion failed".to_string()))?;

        // Create entry header
        let entry = AppendEntry::new(1, data_bytes.len() as u32, checksum);

        // Write entry header as raw bytes
        self.file.write_all(&entry.to_bytes())?;

        // Write data
        self.file.write_all(&data_bytes)?;
        self.file.flush()?;

        Ok(())
    }

    pub fn append_user(&mut self, user: &User) -> Result<(), StorageError> {
        let serializable = SerializableUser::from(user);

        // Serialize user data
        let data_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&serializable)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        // Calculate checksum with hex decode
        let checksum_str = sha256::digest(data_bytes.as_slice());
        let checksum: [u8; 32] = hex::decode(&checksum_str)
            .map_err(|e| StorageError::Serialization(format!("Hex decode failed: {}", e)))?
            .try_into()
            .map_err(|_| StorageError::Serialization("Checksum conversion failed".to_string()))?;

        // Create entry header
        let entry = AppendEntry::new(2, data_bytes.len() as u32, checksum);

        // Write entry header as raw bytes
        self.file.write_all(&entry.to_bytes())?;

        // Write data
        self.file.write_all(&data_bytes)?;
        self.file.flush()?;

        Ok(())
    }

    pub fn read_all_entries(&mut self) -> Result<Vec<(AppendEntry, Vec<u8>)>, StorageError> {
        let mut entries = Vec::new();

        // Seek to beginning
        self.file.seek(SeekFrom::Start(0))?;

        loop {
            // Read fixed-size entry header
            let mut header_buf = [0u8; ENTRY_HEADER_SIZE];
            match self.file.read_exact(&mut header_buf) {
                Ok(()) => {
                    let entry = AppendEntry::from_bytes(&header_buf)?;

                    // Check magic
                    if entry.magic != APPEND_MAGIC {
                        // Might be padding or EOF, break
                        break;
                    }

                    // Read data
                    let mut data_buf = vec![0u8; entry.data_size as usize];
                    self.file.read_exact(&mut data_buf)?;

                    // Verify checksum with hex decode
                    let computed = sha256::digest(&data_buf);
                    let computed_bytes: [u8; 32] = hex::decode(&computed)
                        .map_err(|_| {
                            StorageError::Deserialization("Checksum conversion failed".to_string())
                        })?
                        .try_into()
                        .map_err(|_| {
                            StorageError::Deserialization("Checksum conversion failed".to_string())
                        })?;

                    if computed_bytes != entry.checksum {
                        return Err(StorageError::ChecksumMismatch);
                    }

                    entries.push((entry, data_buf));
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(entries)
    }

    pub fn truncate(&mut self) -> Result<(), StorageError> {
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    pub fn delete(self) -> Result<(), StorageError> {
        drop(self.file);
        std::fs::remove_file(&self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    #![allow(clippy::indexing_slicing)]
    #![allow(clippy::panic)]
    #![allow(unused_must_use)]

    use super::*;
    use crate::core::{Record, User};
    use std::fs;

    fn cleanup_test_files(base_path: &str) {
        let _ = fs::remove_file(base_path);
        let _ = fs::remove_file(format!("{}.wal", base_path));
    }

    fn create_test_record(index: usize, payload: &str) -> Record {
        let signer = User::new("test_signer");
        Record::new(index, payload, "prev_hash", vec![signer])
    }

    #[test]
    fn test_entry_serialization() {
        let checksum = [0u8; 32];
        let entry = AppendEntry::new(1, 100, checksum);

        let bytes = entry.to_bytes();
        let entry2 = AppendEntry::from_bytes(&bytes).unwrap();

        assert_eq!(entry.magic, entry2.magic);
        assert_eq!(entry.entry_type, entry2.entry_type);
        assert_eq!(entry.timestamp, entry2.timestamp);
        assert_eq!(entry.data_size, entry2.data_size);
        assert_eq!(entry.checksum, entry2.checksum);
    }

    #[test]
    fn test_append_and_read_basic() {
        let test_path = "test_basic_append";
        cleanup_test_files(test_path);

        // Test user append
        {
            let mut append_log = AppendLog::new(test_path).unwrap();
            let user = User::new("test_user");
            append_log.append_user(&user).unwrap();

            let entries = append_log.read_all_entries().unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].0.entry_type, 2);
        }

        cleanup_test_files(test_path);
    }

    #[test]
    fn test_append_record_basic() {
        let test_path = "test_record_append";
        cleanup_test_files(test_path);

        let mut append_log = AppendLog::new(test_path).unwrap();
        let record = create_test_record(1, "Test payload");
        append_log.append_record(&record).unwrap();

        let entries = append_log.read_all_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0.entry_type, 1);

        cleanup_test_files(test_path);
    }

    #[test]
    fn test_append_multiple() {
        let test_path = "test_append_multi";
        cleanup_test_files(test_path);

        let mut append_log = AppendLog::new(test_path).unwrap();

        for i in 0..3 {
            let user = User::new(&format!("user_{}", i));
            append_log.append_user(&user).unwrap();
        }

        let entries = append_log.read_all_entries().unwrap();
        assert_eq!(entries.len(), 3);

        for (i, (entry, _)) in entries.iter().enumerate() {
            assert_eq!(entry.entry_type, 2);
            //timestamps should be increasing (or at least not decreasing)
            if i > 0 {
                assert!(entry.timestamp >= entries[i - 1].0.timestamp);
            }
        }

        cleanup_test_files(test_path);
    }

    #[test]
    fn test_truncate() {
        let test_path = "test_truncate";
        cleanup_test_files(test_path);

        let mut append_log = AppendLog::new(test_path).unwrap();

        let user = User::new("test_user");
        append_log.append_user(&user).unwrap();

        assert_eq!(append_log.read_all_entries().unwrap().len(), 1);

        append_log.truncate().unwrap();

        assert_eq!(append_log.read_all_entries().unwrap().len(), 0);

        let user2 = User::new("test_user2");
        append_log.append_user(&user2).unwrap();
        assert_eq!(append_log.read_all_entries().unwrap().len(), 1);

        cleanup_test_files(test_path);
    }

    #[test]
    fn test_empty_log() {
        let test_path = "test_empty";
        cleanup_test_files(test_path);

        let mut append_log = AppendLog::new(test_path).unwrap();

        let entries = append_log.read_all_entries().unwrap();
        assert!(entries.is_empty());

        cleanup_test_files(test_path);
    }

    #[test]
    fn test_file_persistence_simple() {
        let test_path = "test_persist";
        cleanup_test_files(test_path);

        // Write data
        {
            let mut append_log = AppendLog::new(test_path).unwrap();
            append_log
                .append_user(&User::new("persistent_user"))
                .unwrap();
        }

        // Reopen and read
        {
            let mut append_log = AppendLog::new(test_path).unwrap();
            let entries = append_log.read_all_entries().unwrap();
            assert_eq!(entries.len(), 1);
        }

        cleanup_test_files(test_path);
    }
    #[test]
    fn test_checksum_failure() {
        let test_path = "test_checksum";
        cleanup_test_files(test_path);

        let mut append_log = AppendLog::new(test_path).unwrap();

        append_log.append_user(&User::new("test_user")).unwrap();

        drop(append_log);

        // Reopen file nd corrupt the stored checksum
        let wal_path = format!("{}.wal", test_path);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&wal_path)
            .unwrap();

        // Read the header
        let mut header = [0u8; ENTRY_HEADER_SIZE];
        file.read_exact(&mut header).unwrap();

        // Corrupt one byte of the checksum (checksum starts at byte 17)
        header[17] = header[17].wrapping_add(1);

        // Write back the corrupted header
        file.seek(SeekFrom::Start(0)).unwrap();
        file.write_all(&header).unwrap();
        file.flush().unwrap();

        let mut append_log = AppendLog::new(test_path).unwrap();
        let result = append_log.read_all_entries();

        // Should fail because checksum in header doesn't match computed checksum of data
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                StorageError::ChecksumMismatch => {
                    println!("Got expected ChecksumMismatch error");
                }
                _ => panic!("Expected ChecksumMismatch, got {:?}", e),
            }
        }

        cleanup_test_files(test_path);
    }

    #[test]
    fn test_mixed_entries() {
        let test_path = "test_mixed";
        cleanup_test_files(test_path);

        let mut append_log = AppendLog::new(test_path).unwrap();

        // Append mixed entries
        let user = User::new("user1");
        let record = create_test_record(1, "Payload 1");

        append_log.append_user(&user).unwrap();
        append_log.append_record(&record).unwrap();
        append_log.append_user(&User::new("user2")).unwrap();

        let entries = append_log.read_all_entries().unwrap();
        assert_eq!(entries.len(), 3);

        // Check order and types
        assert_eq!(entries[0].0.entry_type, 2); // User
        assert_eq!(entries[1].0.entry_type, 1); // Record  
        assert_eq!(entries[2].0.entry_type, 2); // User
        cleanup_test_files(test_path);
    }

    #[test]
    fn test_large_data() {
        let test_path = "test_large";
        cleanup_test_files(test_path);

        let mut append_log = AppendLog::new(test_path).unwrap();

        let large_payload = "x".repeat(1000); // 1KB payload
        let record = create_test_record(1, &large_payload);

        append_log.append_record(&record).unwrap();

        let entries = append_log.read_all_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].0.data_size > 1000); // Should be larger than raw payload due to serialization

        cleanup_test_files(test_path);
    }
}
