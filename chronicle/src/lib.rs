pub mod utility;

mod window;
use window::Window;

mod vk_instance;
use vk_instance::VkInstance;
mod vk_device;
mod vk_swapchain;

pub struct App {
    window: Window,
    vk_instance: VkInstance
}

impl App {
    pub fn new(title: &'static str, width: u32, height: u32) -> Self {
        let window = Window::new(title, width, height);
        let vk_instance = VkInstance::new(title, &window);

        App {
            window: window,
            vk_instance: vk_instance
        }
    }

    pub fn run(self) {
        self.window.main_loop();
    }
}