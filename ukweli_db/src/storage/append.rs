use rkyv::{Archive, Deserialize, Serialize};

use rkyv::rancor::Error as RkyvError;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::core::Record;

use crate::error::StorageError;
use crate::storage::persitence::SerializableRecord;
const APPEND_MAGIC: [u8; 4] = [0x41, 0x50, 0x4E, 0x44]; // "APND" //diff magic cuz its for WAL
const ENTRY_HEADER_SIZE: usize = 128;

#[derive(Archive, Serialize, Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct AppendEntry {
    pub magic: [u8; 4],
    pub entry_type: u8,
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
}

pub struct AppendLog {
    path: PathBuf,
    file: File,
}

impl AppendLog {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, StorageError> {
        let mut append_path = PathBuf::from(db_path.as_ref());
        append_path.set_extension("wal");

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

        let data_bytes = rkyv::to_bytes::<RkyvError>(&serializable)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        let checksum_str = sha256::digest(data_bytes.as_slice());
        let checksum: [u8; 32] = hex::decode(&checksum_str)
            .map_err(|e| StorageError::Serialization(format!("Hex decode failed: {}", e)))?
            .try_into()
            .map_err(|_| StorageError::Serialization("Checksum conversion failed".to_string()))?;

        let entry = AppendEntry::new(1, data_bytes.len() as u32, checksum);

        let entry_bytes = rkyv::to_bytes::<RkyvError>(&entry)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        if entry_bytes.len() > ENTRY_HEADER_SIZE {
            return Err(StorageError::Serialization(
                "Entry header too large".to_string(),
            ));
        }

        self.file.write_all(&entry_bytes)?;
        let padding_needed = ENTRY_HEADER_SIZE.saturating_sub(entry_bytes.len());
        if padding_needed > 0 {
            let padding = vec![0u8; padding_needed];
            self.file.write_all(&padding)?;
        }

        self.file.write_all(&data_bytes)?;
        self.file.flush()?;

        Ok(())
    }
}
