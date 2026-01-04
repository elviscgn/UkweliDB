
use ed25519_dalek::Signature;
use sha256::digest;

#[derive(Clone, Debug)]
pub struct Record {
    pub index: usize,
    pub payload: String,
    pub hash: String,
    pub prev_hash: String,
    pub signers: Vec<String>,
    pub signatures: Vec<Signature>
}

impl Record {
    pub fn new(index: usize, payload: &str, prev_hash: &str) -> Self {
        Self {
            index,
            payload: payload.to_string(),
            hash: digest(payload),
            prev_hash: prev_hash.to_string(),
            signatures: vec![],
            signers: vec![]
        }
    }
}

