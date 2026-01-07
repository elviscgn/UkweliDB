use crate::error::WorkflowError;
use crate::workflow;

use super::definition::Workflow;
use super::validators::Validator;

use std::collections::HashMap;

use serde_json::Value;

pub struct Engine {
    pub workflows: Vec<Workflow>,
    pub validators: HashMap<String, Validator>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            workflows: Vec::new(),
            validators: HashMap::new(),
        }
    }

    fn load_workflow(
        &mut self,
        workflow_map: HashMap<String, Value>,
    ) -> Result<Workflow, WorkflowError> {
        let workflow_json = serde_json::to_value(workflow_map).map_err(|e| {
            WorkflowError::Definition(format!("Failed to serialize workflow JSON: {}", e))
        })?;

        let workflow: Workflow = serde_json::from_value(workflow_json).map_err(|e| {
            WorkflowError::Definition(format!("Failed to deserialize workflow: {}", e))
        })?;

        // workflow.validate(&self.validators)?;

        Ok(workflow)
    }
}
