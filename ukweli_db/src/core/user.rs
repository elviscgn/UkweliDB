use std::collections::HashSet;

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey, ed25519::signature::SignerMut};
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
}
