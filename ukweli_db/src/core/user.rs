use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey, ed25519::signature::SignerMut};
use rand::rngs::OsRng;

#[derive(Clone, Debug)]
pub struct User {
    pub user_id: String,
    signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
    // roles [coming soon]
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
        }
    }

    pub fn sign(&self, data: &[u8]) -> Signature {
        Signer::sign(&self.signing_key, data)
    }
}
