use serde::{Deserialize, Serialize};

use super::state::WorkflowState;
use super::transition::Transition;

use crate::error::WorkflowError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub states: Vec<WorkflowState>,
    pub transitions: Vec<Transition>,
    pub initial_state: String,
}

impl Workflow {
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        states: Vec<WorkflowState>,
        transitions: Vec<Transition>,
        initial_state: &str,
    ) -> Result<Self, WorkflowError> {
        if states.is_empty() {
            return Err(WorkflowError::Definition(
                "Workflow must have at least one state".to_string(),
            ));
        }

        if !states.iter().any(|s| s.id == initial_state) {
            return Err(WorkflowError::Definition(
                "Initial state must be one of the defined states".to_string(),
            ));
        }

        Ok(Workflow {
            id: id.to_owned(),
            name: name.to_owned(),
            description: description.to_owned(),
            states,
            transitions,
            initial_state: initial_state.to_owned(),
        })
    }
}
