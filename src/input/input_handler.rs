#![allow(dead_code)]

use cgmath::Point2;
use std::collections::HashSet;

use miniquad::MouseButton as QuadMouseButton;

#[derive(Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Button4,
    Button5,
}

impl From<QuadMouseButton> for MouseButton {
    fn from(button: QuadMouseButton) -> MouseButton {
        match button {
            QuadMouseButton::Left => MouseButton::Left,
            QuadMouseButton::Right => MouseButton::Right,
            QuadMouseButton::Middle => MouseButton::Middle,
            QuadMouseButton::Unknown => MouseButton::Button4,
        }
    }
}

pub struct InputHandler {
    pub keys: HashSet<String>,
    pub frame_keys: HashSet<String>,
    pub mouse_position: Point2<f64>,
    pub mouse_keys: HashSet<MouseButton>,
    pub wheel: f32,
}

impl InputHandler {
    pub fn new() -> InputHandler {
        InputHandler {
            keys: HashSet::new(),
            frame_keys: HashSet::new(),
            mouse_position: Point2::new(0., 0.),
            mouse_keys: HashSet::new(),
            wheel: 0.,
        }
    }

    pub fn handle_mouse_move(&mut self, mouse_x: i32, mouse_y: i32) {
        let mouse = Point2::new(mouse_x as f64, mouse_y as f64);

        self.mouse_position = mouse;
    }

    pub fn handle_key_down(&mut self, key: String) {
        self.keys.insert(key.clone());
        self.frame_keys.insert(key.clone());
    }

    pub fn handle_end_frame(&mut self) {
        self.frame_keys.clear();
        self.wheel = 0.;
    }

    pub fn handle_key_up(&mut self, key: String) {
        self.keys.remove(&key);
    }

    pub fn handle_mouse_wheel(&mut self, delta_y: f64) {
        self.wheel = delta_y as f32;
    }

    pub fn is_key_pressed(&self, key: &str) -> bool {
        self.keys.contains(key)
    }

    pub fn is_key_down(&self, key: &str) -> bool {
        self.frame_keys.contains(key)
    }

    pub fn is_mouse_key_down(&self, key: &MouseButton) -> bool {
        self.mouse_keys.contains(key)
    }
}
