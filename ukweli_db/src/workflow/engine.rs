use crate::core::User;
use crate::error::WorkflowError;
use crate::workflow::Transition;

use super::definition::Workflow;
use super::validators::Validator;

use std::collections::HashMap;

use serde_json::Value;

pub struct Engine {
    pub workflows: HashMap<String, Workflow>,
    pub validators: HashMap<String, Validator>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            validators: HashMap::new(),
        }
    }

    pub fn load_workflow(
        &mut self,
        workflow_map: HashMap<String, Value>,
    ) -> Result<Workflow, WorkflowError> {
        let workflow_json = serde_json::to_value(workflow_map).map_err(|e| {
            WorkflowError::Parsing(format!("Failed to serialize workflow JSON: {}", e))
        })?;

        let workflow: Workflow = serde_json::from_value(workflow_json).map_err(|e| {
            WorkflowError::Parsing(format!("Failed to deserialize workflow: {}", e))
        })?;

        if workflow.states.is_empty() {
            return Err(WorkflowError::Definition(
                "Workflow must have at least one state".to_string(),
            ));
        }

        let state_ids = workflow
            .states
            .iter()
            .map(|state| state.id.clone())
            .collect::<Vec<String>>();

        if !state_ids.contains(&workflow.initial_state) {
            return Err(WorkflowError::Definition(format!(
                "Start state '{}' not found in workflow states",
                workflow.initial_state
            )));
        }

        self.workflows.insert(workflow.id.clone(), workflow.clone());
        Ok(workflow)
    }

    pub fn load_workflow_from_json(
        &mut self,
        workflow_json: Value,
    ) -> Result<Workflow, WorkflowError> {
        let workflow: Workflow = serde_json::from_value(workflow_json)
            .map_err(|e| WorkflowError::Parsing(format!("Failed to parse workflow: {}", e)))?;

        self.workflows.insert(workflow.id.clone(), workflow.clone());

        Ok(workflow)
    }

    pub fn get_valid_transitions(
        &self,
        workflow_id: &str,
        current_state: &str,
    ) -> Result<Vec<Transition>, WorkflowError> {
        let workflow = self
            .workflows
            .get(workflow_id)
            .ok_or_else(|| WorkflowError::Parsing(format!("Unknown workflow {}", workflow_id)))?;

        let transitions: Vec<Transition> = workflow
            .transitions
            .iter()
            .filter(|t| t.from_state == current_state)
            .cloned()
            .collect();

        Ok(transitions)
    }

    pub fn validate_transition(
        &self,
        workflow_id: &str,
        from_state: &str,
        to_state: &str,
        signers: Vec<User>,
        payload: &str,
    ) -> Result<bool, WorkflowError> {
        let workflow = self
            .workflows
            .get(workflow_id)
            .ok_or_else(|| WorkflowError::Parsing(format!("Unknown workflow {}", workflow_id)))?;

        let transition = workflow
            .transitions
            .iter()
            .find(|t| t.from_state == from_state && t.to_state == to_state)
            .ok_or_else(|| {
                WorkflowError::Validation(format!(
                    "No valid transition from {} to {}",
                    from_state, to_state
                ))
            })?;

        let role_validator = Validator::HasRole {
            required_roles: transition.required_roles.clone(),
        };

        role_validator.validate(payload, signers.clone())?;

        // other validations shld go here

        Ok(true)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // only in tests :) I want them to panic here but never during runtime
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    #![allow(clippy::indexing_slicing)]
    #![allow(clippy::panic)]
    #![allow(clippy::unreachable)]
    #![allow(clippy::assertions_on_result_states)]

    use ed25519_dalek::ed25519::Error;
    use serde_json::json;

    use crate::{State, workflow};

    use super::*;

    #[test]
    fn test_workflow_empty_states() {
        let workflow = Workflow::new("test_0", "Test", "testtt", vec![], vec![], "");

        assert!(workflow.is_err())
    }

    #[test]
    fn test_workflow_state_not_in_states() {
        let states = vec![State {
            id: "s1".to_string(),
            label: "state 1".to_string(),
        }];

        let transitions: Vec<Transition> = vec![];

        let workflow = Workflow::new("test_0", "Test", "testtt", states, transitions, "s2");

        assert!(workflow.is_err());
    }

    fn create_test_workflow() -> HashMap<String, Value> {
        let workflow = json!({
          "workflow_id": "land_registry_v1",
          "name": "Land Registry",
          "description": "Immutable workflow for property ownership transfers",
          "initial_state": "application",
          "states": [
            {"id": "application", "label": "Application Submitted"},
            {"id": "verification", "label": "Verify Documents"},
            {"id": "approval", "label": "Approve Transfer"},
            {"id": "registered", "label": "Ownership Registered"}
          ],
          "transitions": [
            {
              "from_state": "application",
              "to_state": "verification",
              "name": "submit",
              "required_roles": ["applicant"]
            },
            {
              "from_state": "verification",
              "to_state": "approval",
              "name": "verify",
              "required_roles": ["inspector"]
            },
            {
              "from_state": "approval",
              "to_state": "registered",
              "name": "finalize",
              "required_roles": ["officer", "clerk"]
            }
          ]
        }
        );

        serde_json::from_value(workflow).expect("Failed to create test workflow")
    }

    #[test]
    fn test_workflow_loads() {
        let mut engine = Engine::new();
        let workflow_json = create_test_workflow();

        let result = engine.load_workflow(workflow_json);

        assert!(result.is_ok());

        let workflow = result.unwrap();

        assert_eq!(workflow.id, "land_registry_v1");
        assert_eq!(workflow.states.len(), 4);
        assert_eq!(workflow.transitions.len(), 3);

        let fetched_workflow = engine.workflows.get("land_registry_v1");
        assert!(fetched_workflow.is_some());

        let fetched_workflow = fetched_workflow.unwrap();
        assert_eq!(fetched_workflow.id, "land_registry_v1");
    }
}
