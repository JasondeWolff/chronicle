use ash::vk;

use std::rc::Rc;

pub mod graphics;

pub struct App {
    window: graphics::Window,
    vk_instance: graphics::VkInstance,
    physical_device: graphics::VkPhysicalDevice,
    device: Rc<graphics::VkLogicalDevice>,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain: graphics::VkSwapchain
}

impl App {
    pub fn new(title: &'static str, width: u32, height: u32) -> Self {
        let window = graphics::Window::new(title, width, height);
        let vk_instance = graphics::VkInstance::new(title, &window);

        let physical_device = graphics::VkPhysicalDevice::new(&vk_instance);
        let device = graphics::VkLogicalDevice::new(&vk_instance, &physical_device);
        let graphics_queue = device.get_graphics_queue();
        let present_queue = device.get_present_queue();
        let swapchain = graphics::VkSwapchain::new(
            &vk_instance,
            device.clone(), &physical_device,
            window.width(),
            window.height()
        );
        
        App {
            window: window,
            vk_instance: vk_instance,

            physical_device: physical_device,
            device: device,
            graphics_queue: graphics_queue,
            present_queue: present_queue,
            swapchain: swapchain
        }
    }

    pub fn run(self) {
        self.window.main_loop();
    }
}