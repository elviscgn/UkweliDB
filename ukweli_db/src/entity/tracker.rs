use std::collections::HashMap;

use super::state::EntityState;
use crate::error::EntityError;

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

    pub fn update_entity(
        &mut self,
        entity_id: &str,
        new_state: &str,
        record_index: usize,
    ) -> Result<(), EntityError> {
        // check if entity exists

        let entity = self
            .entities
            .get_mut(entity_id)
            .ok_or(EntityError::Update(format!(
                "Entity with ID {} does not exist",
                entity_id
            )))?;

        entity.current_state = new_state.to_string();
        //timestamp
        entity.last_record_index = record_index;

        Ok(())
    }
}
