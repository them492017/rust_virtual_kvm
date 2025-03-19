use evdev::{EvdevEnum, Key};

const NUM_KEYS: usize = 0x2e7 - 1;

// TODO: move to input-capture crate... should probably include as method in exposed trait
pub struct KeyboardState {
    mapping: [bool; NUM_KEYS],
}

impl KeyboardState {
    pub fn press_key(&mut self, key: Key) {
        self.mapping[key.to_index()] = true
    }

    pub fn release_key(&mut self, key: Key) {
        self.mapping[key.to_index()] = false
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.mapping[key.to_index()]
    }

    pub fn is_combination_pressed(&self, combination: Vec<Key>) -> bool {
        combination.into_iter().all(|key| self.is_key_pressed(key))
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            mapping: [false; NUM_KEYS],
        }
    }
}

pub const CYCLE_TARGET: [Key; 3] = [Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT, Key::KEY_H];
