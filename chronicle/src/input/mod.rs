pub use winit::event::{MouseButton, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use cgmath::Vector2;

const MAX_KEYS: usize = 512;
const MAX_BUTTONS: usize = 32;

pub struct Input {
    keys: [bool; MAX_KEYS],
    keys_prev: [bool; MAX_KEYS],
    buttons: [bool; MAX_BUTTONS],
    buttons_prev: [bool; MAX_BUTTONS],
    mouse_pos: Vector2<i32>,
    mouse_pos_prev: Vector2<i32>
}

impl Input {
    pub(crate) fn init() -> Box<Self> {
        Box::new(Input {
            keys: [false; MAX_KEYS],
            keys_prev: [false; MAX_KEYS],
            buttons: [false; MAX_BUTTONS],
            buttons_prev: [false; MAX_BUTTONS],
            mouse_pos: Vector2::new(0, 0),
            mouse_pos_prev: Vector2::new(0, 0)
        })
    }

    pub(crate) fn update(&mut self) {
        self.keys_prev = self.keys.clone();
        self.buttons_prev = self.buttons.clone();
    }

    pub fn key(&self, key_code: VirtualKeyCode) -> bool {
        self.keys[key_code as usize]
    }

    pub fn key_down(&self, key_code: VirtualKeyCode) -> bool {
        self.keys[key_code as usize] && !self.keys_prev[key_code as usize]
    }

    pub fn mouse_button(&self, button: MouseButton) -> bool {
        self.buttons[Self::mb_to_idx(button)]
    }

    pub fn mouse_button_down(&self, button: MouseButton) -> bool {
        self.buttons[Self::mb_to_idx(button)] && !self.buttons_prev[Self::mb_to_idx(button)]
    }

    pub fn mouse_pos(&self) -> Vector2<i32> {
        self.mouse_pos
    }

    pub fn mouse_delta(&self) -> Vector2<i32> {
        self.mouse_pos - self.mouse_pos_prev
    }

    pub(crate) fn set_key(&mut self, key_code: VirtualKeyCode, value: bool) {
        self.keys[key_code as usize] = value;
    }

    pub(crate) fn set_mouse_button(&mut self, button: MouseButton, value: bool) {
        self.keys[Self::mb_to_idx(button)] = value;
    }

    pub(crate) fn set_mouse_pos(&mut self, mouse_pos: Vector2<i32>) {
        self.mouse_pos = mouse_pos;
    }

    fn mb_to_idx(button: MouseButton) -> usize {
        match button {
            MouseButton::Right => 0,
            MouseButton::Middle => 1,
            MouseButton::Left => 2,
            MouseButton::Other(i) => (3 + i as usize)
        }
    }
}