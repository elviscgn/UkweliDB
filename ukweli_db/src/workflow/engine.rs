use crate::core::User;
use crate::error::WorkflowError;
use crate::workflow::Transition;

use super::definition::Workflow;
use std::collections::HashMap;

use serde_json::Value;

pub struct Engine {
    pub workflows: HashMap<String, Workflow>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
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
        _payload: &str,
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

        let signer_roles: Vec<String> = signers
            .iter()
            .flat_map(|s| s.roles.iter().cloned())
            .collect();
        let missing_roles: Vec<String> = transition
            .required_roles
            .iter()
            .filter(|r| !signer_roles.contains(r))
            .cloned()
            .collect();

        if !missing_roles.is_empty() {
            return Err(WorkflowError::Validation(format!(
                "Missing required roles: {:?}",
                missing_roles
            )));
        }

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

    use serde_json::json;

    use crate::WorkflowState;

    use super::*;

    #[test]
    fn test_workflow_empty_states() {
        let workflow = Workflow::new("test_0", "Test", "testtt", vec![], vec![], "");

        assert!(workflow.is_err())
    }

    #[test]
    fn test_workflow_state_not_in_states() {
        let states = vec![WorkflowState {
            id: "s1".to_string(),
            label: "state 1".to_string(),
        }];

        let transitions: Vec<Transition> = vec![];

        let workflow = Workflow::new("test_0", "Test", "testtt", states, transitions, "s2");

        assert!(workflow.is_err());
    }

    fn create_test_workflow() -> HashMap<String, Value> {
        let workflow = json!({
         "id": "test_workflow",
            "name": "Test Workflow",
            "description": "A test workflow",
            "initial_state": "draft",
            "states": [
                {"id": "draft", "label": "Draft"},
                {"id": "review", "label": "Under Review"},
                {"id": "published", "label": "Published"},
                {"id": "archived", "label": "Archived"}
            ],
            "transitions": [
                {
                    "from_state": "draft",
                    "to_state": "review",
                    "name": "Submit for Review",
                    "required_roles": ["editor"],
                },
                {
                    "from_state": "review",
                    "to_state": "published",
                    "name": "Publish",
                    "required_roles": ["admin", "editor"],
                },
                {
                    "from_state": "review",
                    "to_state": "draft",
                    "name": "Return to Draft",
                    "required_roles": ["editor"],
                },
                {
                    "from_state": "published",
                    "to_state": "archived",
                    "name": "Archive",
                    "required_roles": ["admin"],
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

        assert!(
            result.is_ok(),
            "Workflow failed to load: {:?}",
            result.err()
        );

        let workflow = result.unwrap();

        assert_eq!(workflow.id, "test_workflow");
        assert_eq!(workflow.states.len(), 4);
        assert_eq!(workflow.transitions.len(), 4);

        let fetched_workflow = engine.workflows.get("test_workflow");
        assert!(fetched_workflow.is_some());

        let fetched_workflow = fetched_workflow.unwrap();
        assert_eq!(fetched_workflow.id, "test_workflow");
    }

    #[test]
    fn test_get_valid_transitions() {
        let mut engine = Engine::new();
        let workflow_json = create_test_workflow();

        engine.load_workflow(workflow_json).unwrap();

        let transitions = engine
            .get_valid_transitions("test_workflow", "draft")
            .unwrap();

        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].to_state, "review");

        let transitions = engine
            .get_valid_transitions("test_workflow", "review")
            .unwrap();

        assert_eq!(transitions.len(), 2);

        let to_states = transitions
            .iter()
            .map(|t| t.to_state.as_str())
            .collect::<Vec<&str>>();

        assert_eq!(to_states, vec!["published", "draft"]);

        let transitions = engine
            .get_valid_transitions("test_workflow", "archived")
            .unwrap();

        assert_eq!(transitions.len(), 0);
    }

    #[test]
    fn test_validate_transition_success() {
        let mut engine = Engine::new();
        let workflow_json = create_test_workflow();

        engine.load_workflow(workflow_json).unwrap();

        let mut editor_user = User::new("user_editor");
        editor_user.add_role("editor");

        let result = engine
            .validate_transition(
                "test_workflow",
                "draft",
                "review",
                vec![editor_user],
                "hmmm",
            )
            .unwrap();

        assert!(result)
    }

    #[test]
    fn test_validate_transition_missing_role() {
        let mut engine = Engine::new();
        let workflow_json = create_test_workflow();

        engine.load_workflow(workflow_json).unwrap();

        let editor_user = User::new("user_editor");
        // editor_user.add_role("editor"); no role

        let result = engine.validate_transition(
            "test_workflow",
            "draft",
            "review",
            vec![editor_user],
            "hmmm",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_validation_transition_no_such_transition() {
        let mut engine = Engine::new();
        let workflow_json = create_test_workflow();

        engine.load_workflow(workflow_json).unwrap();

        let mut editor_user = User::new("user_editor");
        editor_user.add_role("editor");

        let result = engine.validate_transition(
            "test_workflow",
            "draft",
            "published", // no such transition
            vec![editor_user],
            "hmmm",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_transition_multiple_signers_combined_roles() {
        let mut engine = Engine::new();
        let workflow_json = create_test_workflow();

        engine.load_workflow(workflow_json).unwrap();

        let mut admin_user = User::new("user_admin");
        admin_user.add_role("admin");

        let mut editor_user = User::new("user_editor");
        editor_user.add_role("editor");

        let result1 = engine.validate_transition(
            "test_workflow",
            "review",
            "published",
            vec![admin_user.clone()],
            "hmmm",
        );

        assert!(result1.is_err());

        let result2 = engine.validate_transition(
            "test_workflow",
            "review",
            "published",
            vec![editor_user.clone()],
            "hmmm",
        );

        assert!(result2.is_err());

        let result3 = engine
            .validate_transition(
                "test_workflow",
                "review",
                "published",
                vec![admin_user, editor_user],
                "hmmm",
            )
            .unwrap();

        assert!(result3);
    }
}
