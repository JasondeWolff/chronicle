pub use winit::event::{MouseButton, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use cgmath::Vector2;

use crate::app;

const MAX_KEYS: usize = 512;
const MAX_BUTTONS: usize = 32;

#[derive(Copy, Clone, Debug)]
pub enum CursorMode {
    FREE,
    LOCKED
}

pub struct Input {
    keys: [bool; MAX_KEYS],
    keys_prev: [bool; MAX_KEYS],
    buttons: [bool; MAX_BUTTONS],
    buttons_prev: [bool; MAX_BUTTONS],
    mouse_pos: Vector2<i32>,
    mouse_delta: Vector2<f32>,
    cursor_mode: CursorMode
}

impl Input {
    pub(crate) fn init() -> Box<Self> {
        Box::new(Input {
            keys: [false; MAX_KEYS],
            keys_prev: [false; MAX_KEYS],
            buttons: [false; MAX_BUTTONS],
            buttons_prev: [false; MAX_BUTTONS],
            mouse_pos: Vector2::new(0, 0),
            mouse_delta: Vector2::new(0.0, 0.0),
            cursor_mode: CursorMode::FREE
        })
    }

    pub(crate) fn update(&mut self) {
        self.keys_prev = self.keys.clone();
        self.buttons_prev = self.buttons.clone();
        self.mouse_delta = Vector2::new(0.0, 0.0);
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

    pub fn mouse_delta(&self) -> Vector2<f32> {
        self.mouse_delta
    }

    pub fn get_cursor_mode(&self) -> CursorMode {
        self.cursor_mode
    }

    pub fn set_cursor_mode(&mut self, mode: CursorMode) {
        let window = app().window().get_winit_window();

        match mode {
            CursorMode::FREE => {
                window.set_cursor_grab(winit::window::CursorGrabMode::None)
                    .expect("Failed to free cursor.");
                window.set_cursor_visible(true);
            },
            CursorMode::LOCKED => {
                let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                    .and_then(|_| {
                        window.set_cursor_grab(winit::window::CursorGrabMode::Locked)
                    });
                window.set_cursor_visible(false);
            }
        }

        self.cursor_mode = mode;
    }

    pub(crate) fn set_key(&mut self, key_code: VirtualKeyCode, value: bool) {
        self.keys[key_code as usize] = value;
    }

    pub(crate) fn set_mouse_button(&mut self, button: MouseButton, value: bool) {
        self.keys[Self::mb_to_idx(button)] = value;

        let imgui = app().graphics().imgui();
        imgui.mouse_button_event(winit_to_imgui_mouse_button(button), value);
    }

    pub(crate) fn set_mouse_pos(&mut self, mouse_pos: Vector2<i32>) {
        self.mouse_pos = mouse_pos;

        let imgui = app().graphics().imgui();
        imgui.mouse_pos_event(mouse_pos.x as f32, mouse_pos.y as f32);
    }

    pub(crate) fn set_mouse_delta(&mut self, mouse_delta: Vector2<f32>) {
        self.mouse_delta = mouse_delta;
    }

    fn mb_to_idx(button: MouseButton) -> usize {
        match button {
            MouseButton::Right => 0,
            MouseButton::Middle => 1,
            MouseButton::Left => 2,
            MouseButton::Other(i) => 3 + i as usize
        }
    }
}

fn winit_to_imgui_mouse_button(button: MouseButton) -> imgui::MouseButton {
    match button {
        MouseButton::Left => imgui::MouseButton::Left,
        MouseButton::Middle => imgui::MouseButton::Middle,
        MouseButton::Right => imgui::MouseButton::Right,
        MouseButton::Other(i) => {
            match i {
                1 => imgui::MouseButton::Extra1,
                _ => imgui::MouseButton::Extra2
            }
        }
    }
}