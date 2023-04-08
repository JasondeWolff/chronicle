use ash::vk;

use std::rc::Rc;

pub mod window;
pub use window::*;

mod vk_device;
use vk_device::*;
mod vk_instance;
use vk_instance::*;
mod vk_shader_module;
use vk_shader_module::*;
mod vk_pipeline;
use vk_pipeline::*;
mod vk_swapchain;
use vk_swapchain::*;
mod utility;
use utility::*;

pub struct Renderer {
    vk_instance: VkInstance,
    physical_device: VkPhysicalDevice,
    device: Rc<VkLogicalDevice>,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    swapchain: VkSwapchain,
    pipeline: VkPipeline
}

impl Renderer {
    pub fn init(window: &Window) -> Box<Self> {
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
        let pipeline = VkPipeline::new(
            device.clone(),
            swapchain.get_extent(),
            &vec![String::from("shader.vert")]//, String::from("shader.frag")]
        );

        Box::new(Renderer {
            vk_instance: vk_instance,

            physical_device: physical_device,
            device: device,
            graphics_queue: graphics_queue,
            present_queue: present_queue,
            swapchain: swapchain,
            pipeline: pipeline
        })
    }

    pub fn update(&mut self) {
        
    }
}