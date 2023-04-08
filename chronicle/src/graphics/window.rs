use crate::CoreLoop;

pub struct Window {
    window: winit::window::Window
}

impl Window {
    pub fn new(core_loop: &CoreLoop, title: &'static str, width: u32, height: u32) -> Self {        
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
            .build(&core_loop.winit_loop())
            .expect("Failed to create window.");

        Window {
            window: window
        }
    }

    pub fn get_winit_window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn width(&self) -> u32 {
        self.window.inner_size().width
    }

    pub fn height(&self) -> u32 {
        self.window.inner_size().height
    }
}