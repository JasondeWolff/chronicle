use ash::vk;
use ash::version::DeviceV1_0;

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
mod vk_fence;
use vk_fence::*;
mod vk_semaphore;
use vk_semaphore::*;
mod utility;
use utility::*;

pub struct Renderer {
    vk_instance: VkInstance,
    physical_device: VkPhysicalDevice,
    device: Rc<VkLogicalDevice>,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    swapchain: Option<VkSwapchain>,
    render_pass: VkRenderPass,
    pipeline: VkPipeline,

    graphics_cmd_pool: VkCmdPool,
    graphics_cmd_buffers: Vec<VkCmdBuffer>
}

impl Renderer {
    pub(crate) fn init(window: &Window) -> Box<Self> {
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
            swapchain.get_extent(), // NOTE!!!!!
            &render_pass,
            &vec![String::from("shader.vert"), String::from("shader.frag")]
        );
        swapchain.build_framebuffers(&render_pass);

        let graphics_cmd_pool = VkCmdPool::new(device.clone());
        let graphics_cmd_buffer = VkCmdBuffer::new(device.clone(), &graphics_cmd_pool, &swapchain);

        Box::new(Renderer {
            vk_instance: vk_instance,
            physical_device: physical_device,
            device: device,
            graphics_queue: graphics_queue,
            present_queue: present_queue,
            swapchain: Some(swapchain),
            render_pass: render_pass,
            pipeline: pipeline,
            graphics_cmd_pool: graphics_cmd_pool,
            graphics_cmd_buffers: graphics_cmd_buffer
        })
    }

    pub(crate) fn update(&mut self) {
        self.render();
    }

    fn render(&mut self) {
        let (img_idx, fence) = self.swapchain.as_ref().unwrap().next_image();
        let img_available = self.swapchain.as_ref().unwrap().image_available_semaphore();
        let render_finished = self.swapchain.as_ref().unwrap().render_finished_semaphore();

        let cmd_buffer = &self.graphics_cmd_buffers[img_idx as usize];
        cmd_buffer.reset();
        cmd_buffer.begin();
        cmd_buffer.begin_render_pass(&self.render_pass, &self.swapchain.as_ref().unwrap(), img_idx as usize);
        cmd_buffer.bind_graphics_pipeline(&self.pipeline);
        cmd_buffer.draw(3, 1, 0, 0);
        cmd_buffer.end_render_pass();
        cmd_buffer.end();

        cmd_buffer.submit(
            &vec![img_available.as_ref()],
            &vec![render_finished.as_ref()],
            fence
        );
        self.swapchain.as_mut().unwrap().present(img_idx, &vec![render_finished.as_ref()]);
    }

    pub(crate) fn wait_idle(&self) {
        unsafe {
            self.device.get_device()
                .device_wait_idle()
                .expect("Failed to wait device idle.")
        };
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.wait_idle();

        self.swapchain = None;
        self.swapchain = Some(VkSwapchain::new(
            &self.vk_instance,
            self.device.clone(), &self.physical_device,
            width, height
        ));
        self.swapchain.as_mut().unwrap().build_framebuffers(&self.render_pass);

    }
}