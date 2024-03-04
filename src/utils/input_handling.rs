use winit::event::*;

pub struct InputHandler {
    pub keys: [u8; 21], // each key is a bit, 0 up, 1 down

    pub mouse_pos: (f64, f64),
    pub prev_mouse_pos: (f64, f64),
    pub mouse_delta: (f64, f64),

    pub callbacks: std::collections::HashMap<VirtualKeyCode, Vec<Box<dyn FnMut()>>>,
}

impl InputHandler {
    pub fn new() -> InputHandler {
        InputHandler {
            keys: [0; 21],
            mouse_pos: (0.0, 0.0),
            prev_mouse_pos: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            callbacks: std::collections::HashMap::new(),
        }
    }

    pub fn handle_key_press(
        &mut self,

        key: VirtualKeyCode,
        state: ElementState,
    ) {
        match state {
            ElementState::Pressed => {
                // Set the key to down
                self.keys[key as usize / 8] |= 1 << (key as usize % 8);

                // Call all the callbacks for the key
                if let Some(callbacks) = self.callbacks.get_mut(&key) { for callback in callbacks { callback(); } }
            },
            ElementState::Released => self.keys[key as usize / 8] &= !(1 << (key as usize % 8)), // Set the key to u{p
        }
    }

    pub fn is_down(&self, key: VirtualKeyCode) -> bool {
        self.keys[key as usize / 8] & (1 << (key as usize % 8)) != 0
    }

    pub fn periodic(&mut self) {
        self.mouse_delta = (self.mouse_pos.0 - self.prev_mouse_pos.0, self.mouse_pos.1 - self.prev_mouse_pos.1);
        self.prev_mouse_pos = self.mouse_pos;
    }

    pub fn register_callback(&mut self, key: VirtualKeyCode, callback: Box<dyn FnMut()>) {
        if let Some(callbacks) = self.callbacks.get_mut(&key) { callbacks.push(callback); }
        else { self.callbacks.insert(key, vec![callback]); }
    }
}
