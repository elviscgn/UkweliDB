use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use ukweli_db::core::User;

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
struct StoredUser {
    user_id: String,

    /// TODO: Encrypt this in production
    /// also prolly have a different way of handling users
    signing_key_bytes: Vec<u8>,
    verifying_key_bytes: Vec<u8>,
    roles: Vec<String>,
}

pub struct UserStore;

impl UserStore {
    pub fn create_user(user_id: &str) -> Result<User> {
        let user = User::new(user_id);
        Self::save_user(&user)?;
        println!("Created user: {}", user_id);
        Ok(user)
    }

    pub fn save_user(user: &User) -> Result<()> {
        let users_dir = Config::users_dir()?;
        std::fs::create_dir_all(&users_dir).context("Failed to create users directory")?;

        let user_file = users_dir.join(format!("{}.json", user.user_id));

        let stored = StoredUser {
            user_id: user.user_id.clone(),
            signing_key_bytes: user.signing_key_bytes().to_vec(),
            verifying_key_bytes: user.verifying_key.to_bytes().to_vec(),
            roles: user.roles.iter().cloned().collect(),
        };

        let content = serde_json::to_string_pretty(&stored)?;
        std::fs::write(&user_file, content).context("Failed to write user file")?;

        Ok(())
    }

    pub fn load_user(user_id: &str) -> Result<User> {
        let users_dir = Config::users_dir()?;
        let user_file = users_dir.join(format!("{}.json", user_id));

        if !user_file.exists() {
            bail!(
                "User '{}' not found. Create with: ukweli user create {}",
                user_id,
                user_id
            );
        }

        let content = std::fs::read_to_string(&user_file).context("Failed to read user file")?;

        let stored: StoredUser =
            serde_json::from_str(&content).context("Failed to parse user file")?;

        let signing_key_bytes: [u8; 32] = stored
            .signing_key_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid signing key length"))?;

        let roles: HashSet<String> = stored.roles.into_iter().collect();

        Ok(User::from_key_bytes(
            &stored.user_id,
            &signing_key_bytes,
            roles,
        ))
    }

    pub fn list_users() -> Result<Vec<String>> {
        let users_dir = Config::users_dir()?;

        if !users_dir.exists() {
            return Ok(vec![]);
        }

        let mut users = vec![];
        for entry in std::fs::read_dir(&users_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let (Some("json"), Some(stem)) = (
                path.extension().and_then(|s| s.to_str()),
                path.file_stem().and_then(|s| s.to_str()),
            ) {
                users.push(stem.to_string());
            }
        }

        Ok(users)
    }

    pub fn delete_user(user_id: &str) -> Result<()> {
        let users_dir = Config::users_dir()?;
        let user_file = users_dir.join(format!("{}.json", user_id));

        if !user_file.exists() {
            bail!("User '{}' not found", user_id);
        }

        std::fs::remove_file(&user_file).context("Failed to delete user file")?;

        println!("Deleted user: {}", user_id);
        Ok(())
    }

    pub fn user_exists(user_id: &str) -> Result<bool> {
        let users_dir = Config::users_dir()?;
        let user_file = users_dir.join(format!("{}.json", user_id));
        Ok(user_file.exists())
    }
}
