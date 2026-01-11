use core::time;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

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

    pub timestamp: u64,
    pub nonce: u64,
}

impl Record {
    #[allow(clippy::expect_used)]
    // Getting timestamp returns a result meaning I would have to propagate the error an errror I can't meaningfully handle
    // if this panics you have bigger issues than a panic hence why i'm using expect here
    pub fn new(index: usize, payload: &str, prev_hash: &str, signers: Vec<User>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System clock is set before UNIX epoch")
            .as_secs();

        let nonce = rand::random();

        let payload_hash = digest(payload);
        let joined_signers = signers
            .iter()
            .map(|u| u.user_id.clone())
            .collect::<Vec<String>>()
            .join(",");

        let material = format!(
            "{} {} {} {} {} {}",
            index, prev_hash, payload_hash, timestamp, nonce, joined_signers
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
            timestamp,
            nonce,
        }
    }
}
