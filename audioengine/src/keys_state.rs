use std::collections::VecDeque;

use types::KeyAction;

pub struct KeysState {
    state: Vec<i32>,
}

impl KeysState {
    pub fn new() -> Self {
        Self { state: Vec::new() }
    }

    pub fn key_down(&mut self, key_action: KeyAction) -> Vec<i32> {
        match key_action {
            KeyAction::Press(value) => {
                self.remove_key(value);
                self.state.push(value);
                self.state.clone()
            }
            KeyAction::Release(value) => {
                self.remove_key(value);
                self.state.clone()
            }
        }
    }

    fn remove_key(&mut self, key: i32) {
        let index_option = self.state.iter().position(|&v| v == key);
        if let Some(index) = index_option {
            self.state.remove(index);
        }
    }
}
