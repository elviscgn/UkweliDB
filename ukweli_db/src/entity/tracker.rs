use std::collections::HashMap;

use super::state::EntityState;

pub struct Tracker {
    pub entities: HashMap<String, EntityState>,
}

impl Tracker {
    pub fn new(
        entity_id: &str,
        workflow_id: &str,
        initial_state: &str,
        record_index: usize,
    ) -> Self {
        let mut entities = HashMap::new();
        entities.insert(
            entity_id.to_string(),
            EntityState::new(entity_id, workflow_id, initial_state, record_index),
        );

        Tracker { entities }
    }
}
