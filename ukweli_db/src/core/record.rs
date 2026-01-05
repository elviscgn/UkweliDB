use std::collections::HashMap;

use ed25519_dalek::Signature;
use sha256::digest;

use crate::core::User;

#[derive(Clone, Debug)]
pub struct Record {
    pub index: usize,
    pub payload: String,
    pub payload_hash: String,

    pub signers: Vec<User>,
    pub signatures: HashMap<String, Signature>,

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

        let record_hash = digest(material);
        let mut record_signatures = HashMap::new();

        for signer in &signers {
            let signature = signer.sign(record_hash.as_bytes());
            record_signatures.insert(signer.clone().user_id, signature);
        }

        Self {
            index,
            payload: payload.to_string(),
            payload_hash,

            signatures: record_signatures,
            signers,

            record_hash,
            prev_hash: prev_hash.to_string(),
        }
    }
}
