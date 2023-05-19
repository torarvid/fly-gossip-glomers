use std::collections::HashSet;

pub struct Node {
    pub id: String,
    messages: HashSet<usize>,
}

impl Node {
    pub fn new(id: String) -> Self {
        Self {
            id,
            messages: HashSet::new(),
        }
    }

    pub fn add_message(&mut self, msg_id: usize) {
        self.messages.insert(msg_id);
    }

    pub fn get_messages(&self) -> &HashSet<usize> {
        &self.messages
    }

    pub fn has_message(&self, msg_id: usize) -> bool {
        self.messages.contains(&msg_id)
    }
}
