pub struct EntityState {
    pub id: String,
    pub workflow_id: String,
    pub current_state: String,
    // state_entered_at: String, undecided yet
    pub last_record_index: String,
}

impl EntityState {
    pub fn new(id: &str, workflow_id: &str, current_state: &str, last_record_index: usize) -> Self {
        Self {
            id: id.to_string(),
            workflow_id: workflow_id.to_string(),
            current_state: current_state.to_string(),
            last_record_index: last_record_index.to_string(),
        }
    }
}
