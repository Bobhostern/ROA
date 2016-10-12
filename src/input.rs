#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Key {
    A, B, C,
    X, Y, Z,
    Start, Select,
    Up, Down, Left, Right,
}

use glium::glutin::{VirtualKeyCode, Event, ElementState};
use std::collections::HashMap;
// Converts events into out virtual key codes
pub struct KeyReader {
    // Keyboard binding
    kbd_map: HashMap<VirtualKeyCode, Key>,
    // TODO Add Touch support
}

impl KeyReader {
    pub fn new() -> KeyReader {
        let mut keymap = HashMap::new();
        keymap.insert(VirtualKeyCode::Up, Key::Up);
        keymap.insert(VirtualKeyCode::Down, Key::Down);
        keymap.insert(VirtualKeyCode::Left, Key::Left);
        keymap.insert(VirtualKeyCode::Right, Key::Right);
        keymap.insert(VirtualKeyCode::Return, Key::Start);
        keymap.insert(VirtualKeyCode::Space, Key::Select);
        keymap.insert(VirtualKeyCode::A, Key::A);
        keymap.insert(VirtualKeyCode::S, Key::B);
        keymap.insert(VirtualKeyCode::D, Key::C);
        keymap.insert(VirtualKeyCode::Z, Key::X);
        keymap.insert(VirtualKeyCode::X, Key::Y);
        keymap.insert(VirtualKeyCode::C, Key::Z);

        KeyReader {
            kbd_map: keymap,
        }
    }

    pub fn interpret_code(&self, e: &VirtualKeyCode) -> Option<Key> {
        self.kbd_map.get(e).cloned()
    }

    pub fn interpret_event(&self, e: &Event) -> Option<Key> {
        match *e {
            Event::KeyboardInput(ElementState::Pressed, _, ref ke) => match *ke {
                Some(k) => self.interpret_code(&k),
                None => None
            },
            _ => None,
        }
    }
}
