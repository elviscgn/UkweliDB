use ed25519_dalek::Signature;
use sha256::digest;

use crate::core::User;

#[derive(Clone, Debug)]
pub struct Record {
    pub index: usize,
    pub payload: String,
    pub payload_hash: String,

    pub signers: Vec<User>,
    pub signatures: Vec<Signature>,

    pub prev_hash: String,
    pub record_hash: String,
}

impl Record {
    pub fn new(index: usize, payload: &str, prev_hash: &str, signers: Vec<User>) -> Self {
        let payload_hash = digest(payload);
        let joined_signers = signers
            .iter()
            .map(|u| u.user_id.clone())
            .collect::<Vec<String>>()
            .join(",");

        let material = format!(
            "{} {} {} {}",
            index, prev_hash, payload_hash, joined_signers
        );

        Self {
            index,
            payload: payload.to_string(),
            payload_hash,

            signatures: vec![],
            signers,

            record_hash: digest(material),
            prev_hash: prev_hash.to_string(),
        }
    }
}
