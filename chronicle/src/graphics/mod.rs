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
mod vk_render_pass;
use vk_render_pass::*;
mod vk_pipeline;
use vk_pipeline::*;
mod vk_swapchain;
use vk_swapchain::*;
mod vk_cmd_pool;
use vk_cmd_pool::*;
mod vk_cmd_buffer;
use vk_cmd_buffer::*;
mod utility;
use utility::*;

pub struct Renderer {
    vk_instance: VkInstance,
    physical_device: VkPhysicalDevice,
    device: Rc<VkLogicalDevice>,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    swapchain: VkSwapchain,
    render_pass: VkRenderPass,
    pipeline: VkPipeline,

    graphics_cmd_pool: VkCmdPool,
    graphics_cmd_buffers: Vec<VkCmdBuffer>
}

impl Renderer {
    pub fn init(window: &Window) -> Box<Self> {
        let vk_instance = VkInstance::new("Chronicle", &window);

        let physical_device = VkPhysicalDevice::new(&vk_instance);
        let device = VkLogicalDevice::new(&vk_instance, &physical_device);
        let graphics_queue = device.get_graphics_queue();
        let present_queue = device.get_present_queue();
        let mut swapchain = VkSwapchain::new(
            &vk_instance,
            device.clone(), &physical_device,
            window.width(), window.height()
        );
        let render_pass = VkRenderPass::new(device.clone(), *swapchain.get_format());
        let pipeline = VkPipeline::new(
            device.clone(),
            swapchain.get_extent(),
            &render_pass,
            &vec![String::from("shader.vert"), String::from("shader.frag")]
        );
        swapchain.build_framebuffers(&render_pass);

        let graphics_cmd_pool = VkCmdPool::new(device.clone());
        let graphics_cmd_buffer = VkCmdBuffer::new(device.clone(), &graphics_cmd_pool);

        for (i, cmd_buffer) in graphics_cmd_buffer.iter().enumerate() {
            cmd_buffer.begin();
            cmd_buffer.begin_render_pass(&render_pass, &swapchain, i);
            cmd_buffer.bind_graphics_pipeline(&pipeline);
            cmd_buffer.draw(3, 1, 0, 0);
            cmd_buffer.end_render_pass();
            cmd_buffer.end();
        }

        Box::new(Renderer {
            vk_instance: vk_instance,
            physical_device: physical_device,
            device: device,
            graphics_queue: graphics_queue,
            present_queue: present_queue,
            swapchain: swapchain,
            render_pass: render_pass,
            pipeline: pipeline,
            graphics_cmd_pool: graphics_cmd_pool,
            graphics_cmd_buffers: graphics_cmd_buffer
        })
    }

    pub fn update(&mut self) {
        
    }
}