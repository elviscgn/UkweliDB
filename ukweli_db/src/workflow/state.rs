use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub id: String,
    pub label: String,
}
