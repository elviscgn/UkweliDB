use std::collections::HashMap;

use super::state::EntityState;

pub struct Tracker {
    entities: HashMap<String, EntityState>,
}
