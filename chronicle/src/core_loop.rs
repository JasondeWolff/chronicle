use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent, DeviceEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use cgmath::Vector2;

use crate::app;

pub struct CoreLoop {
    event_loop: EventLoop<()>
}

impl CoreLoop {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();

        CoreLoop {
            event_loop: event_loop
        }
    }

    pub(crate) fn winit_loop(&self) -> &EventLoop<()> {
        &self.event_loop
    }

    pub fn run(self) {
        self.event_loop.run(move |event, _, control_flow| {
            let app = app();
            if app.is_init() {
                match event {
                    | Event::WindowEvent { event, .. } => {
                        match event {
                            | WindowEvent::CloseRequested => {
                                *control_flow = ControlFlow::Exit
                            },
                            | WindowEvent::Resized(size) => {
                                app.graphics().resize(size.width, size.height);
                            },
                            | WindowEvent::KeyboardInput { input, .. } => {
                                match input {
                                    | KeyboardInput { virtual_keycode, state, .. } => {
                                        match (virtual_keycode, state) {
                                            | (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                                *control_flow = ControlFlow::Exit
                                            },
                                            | (Some(virtual_keycode), state) => {
                                                app.input().set_key(virtual_keycode, state == ElementState::Pressed);
                                            },
                                            | _ => {}
                                        }
                                    },
                                }
                            },
                            | WindowEvent::MouseInput { state, button, .. } => {
                                app.input().set_mouse_button(button, state == ElementState::Pressed);
                            },
                            | WindowEvent::CursorMoved { position, .. } => {
                                app.input().set_mouse_pos(Vector2::new(position.x as i32, position.y as i32));
                            }
                            | _ => {},
                        }
                    },
                    | Event::MainEventsCleared => {
                        app.window().get_winit_window().request_redraw();
                    },
                    | Event::RedrawRequested(_window_id) => {
                        app.update();
                    },
                    | Event::LoopDestroyed => {
                        app.graphics().wait_idle();
                    },
                    | Event::DeviceEvent { event, ..} => {
                        match event {
                            | DeviceEvent::MouseMotion { delta } => {
                                app.input().set_mouse_delta(Vector2::new(delta.0 as f32, delta.1 as f32));
                            },
                            | _ => {}
                        }
                    },
                    _ => (),
                }
            } else {
                app.update();
            }
        })
    }
}