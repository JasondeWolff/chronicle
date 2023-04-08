use ash::vk;

use std::rc::Rc;
use std::cell::RefCell;

mod vk_device;
use vk_device::*;
mod vk_instance;
use vk_instance::*;
mod vk_shader_module;
use vk_shader_module::*;
mod vk_swapchain;
use vk_swapchain::*;
mod window;
use window::*;
mod utility;
use utility::*;

use crate::CoreLoop;

pub struct Renderer {
    window: Window,
    vk_instance: VkInstance,
    physical_device: VkPhysicalDevice,
    device: Rc<VkLogicalDevice>,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain: VkSwapchain
}

impl Renderer {
    pub fn init(core_loop: &CoreLoop) -> Box<Self> {
        let window = Window::new(core_loop, "Chronicle", 1280, 720);
        let vk_instance = VkInstance::new("Chronicle", &window);

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

        Box::new(Renderer {
            window: window,
            vk_instance: vk_instance,

            physical_device: physical_device,
            device: device,
            graphics_queue: graphics_queue,
            present_queue: present_queue,
            swapchain: swapchain
        })
    }

    pub fn update(&mut self) {
        
    }
}

impl Renderer {
    pub(crate) fn get_window(&self) -> &Window {
        &self.window
    }
}