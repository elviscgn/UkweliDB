use rkyv::bytecheck::CheckBytes;
// use rkyv::ser::serializers::AllocSerializer;
use crate::core::User;
use rkyv::{Archive, Deserialize, Serialize};

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
