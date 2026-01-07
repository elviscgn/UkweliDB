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

    fn load_workflow(&mut self, workflow: HashMap<String, Value>) {}
}
