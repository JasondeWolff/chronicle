use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

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
            match event {
                | Event::WindowEvent { event, .. } => {
                    match event {
                        | WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit
                        },
                        | WindowEvent::KeyboardInput { input, .. } => {
                            match input {
                                | KeyboardInput { virtual_keycode, state, .. } => {
                                    match (virtual_keycode, state) {
                                        | (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                            *control_flow = ControlFlow::Exit
                                        },
                                        | _ => {},
                                    }
                                },
                            }
                        },
                        | _ => {},
                    }
                },
                | Event::MainEventsCleared => {
                    app().window().get_winit_window().request_redraw();
                },
                | Event::RedrawRequested(_window_id) => {
                    app().update();
                },
                _ => (),
            }
        })
    }
}