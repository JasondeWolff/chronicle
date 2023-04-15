use ash::vk;

use std::rc::Rc;

pub use super::window::*;

pub mod vk_device;
pub use vk_device::*;
pub mod vk_instance;
pub use vk_instance::*;
pub mod vk_shader_module;
pub use vk_shader_module::*;
pub mod vk_render_pass;
pub use vk_render_pass::*;
pub mod vk_pipeline;
pub use vk_pipeline::*;
pub mod vk_swapchain;
pub use vk_swapchain::*;
pub mod vk_cmd_pool;
pub use vk_cmd_pool::*;
pub mod vk_cmd_buffer;
pub use vk_cmd_buffer::*;
pub mod vk_cmd_queue;
pub use vk_cmd_queue::*;
pub mod vk_fence;
pub use vk_fence::*;
pub mod vk_semaphore;
pub use vk_semaphore::*;
pub mod utility;
pub mod vk_vertex;
pub use vk_vertex::*;
pub mod buffers;
pub use buffers::*;
pub mod vk_mesh;
pub use vk_mesh::*;
pub mod descriptors;
pub use descriptors::*;
pub mod vk_sampler;
pub use vk_sampler::*;

use crate::graphics::*;

pub struct VkApp {
    vk_instance: VkInstance,
    physical_device: VkPhysicalDevice,
    device: Rc<VkLogicalDevice>,
    graphics_queue: VkCmdQueue,
    present_queue: VkCmdQueue,
    swapchain: Option<RcCell<VkSwapchain>>,
    desc_pool: Rc<VkDescriptorPool>
}

impl VkApp {
    pub fn new(window: &Window) -> Self {
        let vk_instance = VkInstance::new("Chronicle", &window);
        let physical_device = VkPhysicalDevice::new(&vk_instance);
        let device = VkLogicalDevice::new(&vk_instance, &physical_device);

        let descriptor_pool = VkDescriptorPool::new(device.clone());

        let graphics_queue = VkCmdQueue::new(
            device.clone(),
            descriptor_pool.clone(),
            device.get_graphics_queue(),
            VkQueueType::GRAPHICS
        );
        let present_queue = VkCmdQueue::new(
            device.clone(),
            descriptor_pool.clone(),
            device.get_present_queue(),
            VkQueueType::PRESENT
        );

        let swapchain = VkSwapchain::new(
            &vk_instance,
            device.clone(), &physical_device,
            window.width(), window.height()
        );

        VkApp {
            vk_instance: vk_instance,
            physical_device: physical_device,
            device: device,
            graphics_queue: graphics_queue,
            present_queue: present_queue,
            swapchain: Some(swapchain),
            desc_pool: descriptor_pool
        }
    }

    pub fn update(&mut self) {
        self.graphics_queue.process_busy_cmds();
        self.present_queue.process_busy_cmds();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.device.wait_idle();

        self.swapchain = None;
        if width > 0 && height > 0 {
            self.swapchain = Some(VkSwapchain::new(
                &self.vk_instance,
                self.device.clone(), &self.physical_device,
                width, height
            ));
        }
    }

    pub fn get_device(&self) -> Rc<VkLogicalDevice> {
        self.device.clone()
    }

    pub fn get_physical_device(&self) -> &VkPhysicalDevice {
        &self.physical_device
    }

    pub fn get_cmd_queue(&mut self) -> &mut VkCmdQueue {
        &mut self.graphics_queue
    }

    pub fn get_swapchain(&self) -> Option<RcCell<VkSwapchain>> {
        match &self.swapchain {
            Some(swapchain) => Some(swapchain.clone()),
            None => None
        }
    }

    pub fn get_desc_pool(&self) -> Rc<VkDescriptorPool> {
        self.desc_pool.clone()
    }
}