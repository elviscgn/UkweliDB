
use ed25519_dalek::Signature;
use sha256::digest;

#[derive(Clone, Debug)]
pub struct Record {
    pub index: usize,
    pub payload: String,
    pub payload_hash: String,
    
    pub signers: Vec<String>,
    pub signatures: Vec<Signature>,

    pub prev_hash: String,
    pub record_hash: String,
}

impl Record {
    pub fn new(index: usize, payload: &str, prev_hash: &str) -> Self {
        Self {
            index,
            payload: payload.to_string(),
            payload_hash: digest(payload),
            
            signatures: vec![],
            signers: vec![],

            record_hash: "".to_string(),
            prev_hash: prev_hash.to_string(),

            
        }
    }
}

