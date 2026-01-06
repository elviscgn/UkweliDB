use super::definition::Workflow;
use super::validators::Validator;

use std::collections::HashMap;
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
}