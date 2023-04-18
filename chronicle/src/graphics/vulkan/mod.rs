use std::{rc::Rc, collections::HashMap};

use crate::Window;

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

pub trait ToAny: 'static {
    fn as_any(&mut self) -> &mut dyn std::any::Any;
}

pub struct VkApp {
    instance: VkInstance,
    physical_device: VkPhysicalDevice,
    device: Rc<VkLogicalDevice>,
    graphics_queue: VkCmdQueue,
    present_queue: VkCmdQueue,
    swapchain: Option<RcCell<VkSwapchain>>,
    desc_pool: Rc<VkDescriptorPool>,

    uniform_buffers: HashMap<String, Vec<RcCell<VkUniformBuffer>>>
}

impl VkApp {
    pub fn new(window: &Window) -> Self {
        let instance = VkInstance::new("Chronicle", &window);
        let physical_device = VkPhysicalDevice::new(&instance);
        let device = VkLogicalDevice::new(&instance, &physical_device);

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
            &instance,
            device.clone(), &physical_device,
            window.width(), window.height()
        );

        VkApp {
            instance: instance,
            physical_device: physical_device,
            device: device,
            graphics_queue: graphics_queue,
            present_queue: present_queue,
            swapchain: Some(swapchain),
            desc_pool: descriptor_pool,
            uniform_buffers: HashMap::new()
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
                &self.instance,
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

    pub fn uniform_buffer<T: ToAny>(&mut self, name: &str) -> RcCell<VkUniformBuffer> {
        let name = String::from(name);

        let swapchain = self.swapchain.as_ref().unwrap().as_ref();
        let img_count = swapchain.get_framebuffer_count() as usize;
        let img_idx = swapchain.get_current_img() as usize;

        match self.uniform_buffers.get(&name) {
            Some(uniform_buffer) => uniform_buffer[img_idx].clone(),
            None => {
                let mut uniform_buffers = Vec::new();
                for _ in 0..img_count {
                    uniform_buffers.push(RcCell::new(VkUniformBuffer::new::<T>(
                        self.device.clone(),
                        &self.physical_device
                    )));
                }

                let uniform_buffer = uniform_buffers[img_idx].clone();
                self.uniform_buffers.insert(name, uniform_buffers);
                uniform_buffer
            }
        }
    }
}