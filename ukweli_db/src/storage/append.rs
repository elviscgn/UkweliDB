use rkyv::{Archive, Deserialize, Serialize};

// const APPEND_MAGIC: [u8; 4] = [0x41, 0x50, 0x4E, 0x44]; // "APND" //diff magic cuz its for WAL
// const ENTRY_HEADER_SIZE: usize = 128;

#[derive(Archive, Serialize, Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct AppendEntry {
    pub magic: [u8; 4],
    pub entry_type: u8,
    pub timestamp: u64,
    pub data_size: u32,
    pub checksum: [u8; 32],
}
