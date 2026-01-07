use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from_state: String,
    pub to_state: String,
    pub name: String,
    pub required_roles: Vec<String>,
    
}