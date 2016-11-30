// TODO Make the double-texture rendering facility as seen in main_state.rs a utility.
use std::collections::HashMap;
use input::Key;

pub struct KeyTracker {
    keymap: HashMap<Key, bool>,
}

impl KeyTracker {
    pub fn new() -> KeyTracker {
        KeyTracker {
            keymap: HashMap::new()
        }
    }

    pub fn pressed(&mut self, k: Key) {
        *self.keymap.entry(k).or_insert(true) = true
    }

    pub fn released(&mut self, k: Key) {
        *self.keymap.entry(k).or_insert(false) = false;
    }

    pub fn held_keys(&self) -> Vec<Key> {
        self.keymap.iter().filter(|&(_, v)| *v).map(|(k, _)| k.clone()).collect()
    }
}
