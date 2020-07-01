use gl_wrapper::unwrap_option_or_ret;
use gl_wrapper::unwrap_result_or_ret;
use glutin::event::VirtualKeyCode;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

pub struct KeyboardHandler {
    keyb: Vec<bool>,
    map: HashMap<glutin::event::VirtualKeyCode, u32>,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        KeyboardHandler {
            keyb: vec![false; 128],
            map: HashMap::with_capacity(128),
        }
    }

    pub fn handle(self: &mut Self, key_ev: &glutin::event::KeyboardInput) {
        match key_ev {
            glutin::event::KeyboardInput {
                scancode: x,
                state: glutin::event::ElementState::Pressed,
                virtual_keycode: v,
                ..
            } => {
                let i: usize = (*x).try_into().unwrap();
                if i >= self.keyb.len() {
                    self.keyb.resize(i + 1, false);
                }
                self.keyb[i] = true;
                self.map.insert(v.unwrap(), *x);
            }
            glutin::event::KeyboardInput {
                scancode: x,
                state: glutin::event::ElementState::Released,
                virtual_keycode: _,
                ..
            } => {
                let i: usize = (*x).try_into().unwrap();
                if i >= self.keyb.len() {
                    self.keyb.resize(i + 1, false);
                }
                self.keyb[i] = false;
            }
        }
    }

    pub fn is_pressed(self: &Self, code: VirtualKeyCode) -> Option<bool> {
        self.keyb
            .get(unwrap_result_or_ret!(
                usize::try_from(unwrap_option_or_ret!(self.map.get(&code), None).clone()),
                None
            ))
            .cloned()
    }
}
