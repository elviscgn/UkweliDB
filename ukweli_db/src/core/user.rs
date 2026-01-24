use std::collections::HashSet;

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

#[derive(Clone, Debug)]
pub struct User {
    pub user_id: String,
    signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
    pub roles: HashSet<String>,
}

impl User {
    pub fn new(user_id: &str) -> Self {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        User {
            user_id: user_id.to_owned(),
            signing_key,
            verifying_key,
            roles: HashSet::new(),
        }
    }

    pub fn sign(&self, data: &[u8]) -> Signature {
        Signer::sign(&self.signing_key, data)
    }

    pub fn add_role(&mut self, role: &str) {
        self.roles.insert(role.to_string());
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    pub fn remove_role(&mut self, role: &str) {
        self.roles.remove(role);
    }

    // export priv keys as bytes
    pub fn signing_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    pub fn from_key_bytes(
        user_id: &str,
        signing_key_bytes: &[u8; 32],
        roles: HashSet<String>,
    ) -> Self {
        let signing_key = SigningKey::from_bytes(signing_key_bytes);
        let verifying_key = signing_key.verifying_key();

        User {
            user_id: user_id.to_owned(),
            signing_key,
            verifying_key,
            roles,
        }
    }

    // create a readonly user for verifying only
    pub fn from_verifying_key(
        user_id: &str,
        verifying_key_bytes: &[u8; 32],
        roles: HashSet<String>,
    ) -> Result<Self, ed25519_dalek::SignatureError> {
        let verifying_key = VerifyingKey::from_bytes(verifying_key_bytes)?;

        let signing_key = SigningKey::from_bytes(&[0u8; 32]);

        Ok(User {
            user_id: user_id.to_owned(),
            signing_key,
            verifying_key,
            roles,
        })
    }
}
