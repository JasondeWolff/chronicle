use ash::vk;

use std::rc::Rc;

pub mod utility;

mod window;
use window::Window;

mod vk_instance;
use vk_instance::VkInstance;
mod vk_device;
mod vk_swapchain;

use crate::vk_device::{VkPhysicalDevice, VkLogicalDevice};
use crate::vk_swapchain::VkSwapchain;

pub struct App {
    window: Window,
    vk_instance: VkInstance,
    physical_device: VkPhysicalDevice,
    device: Rc<VkLogicalDevice>,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain: VkSwapchain
}

impl App {
    pub fn new(title: &'static str, width: u32, height: u32) -> Self {
        let window = Window::new(title, width, height);
        let vk_instance = VkInstance::new(title, &window);

        let physical_device = VkPhysicalDevice::new(&vk_instance);
        let device = VkLogicalDevice::new(&vk_instance, &physical_device);
        let graphics_queue = device.get_graphics_queue();
        let present_queue = device.get_present_queue();
        let swapchain = VkSwapchain::new(
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