use crate::{core::user::User, error::WorkflowError};

#[derive(Debug, Clone)]
pub enum Validator {
    AlwaysTrue, // hmmm not every state would need validation
    // HasField => for the future when I make payloads json based
    HasRole { required_roles: Vec<String> },
}

impl Validator {
    pub fn validate(&self, payload: &str, signers: Vec<User>) -> Result<bool, WorkflowError> {
        match self {
            Validator::AlwaysTrue => Ok(true),
            Validator::HasRole { required_roles } => {
                let signer_roles: Vec<String> = signers
                    .iter()
                    .flat_map(|s| s.roles.iter().cloned())
                    .collect();

                let missing_roles: Vec<String> = required_roles
                    .iter()
                    .filter(|role| !signer_roles.contains(role))
                    .cloned()
                    .collect();

                if missing_roles.is_empty() {
                    Ok(true)
                } else {
                    Err(WorkflowError::Validation(format!(
                        "Missing required roles: {:?}",
                        missing_roles
                    )))
                }
            }
        }
    }
}


