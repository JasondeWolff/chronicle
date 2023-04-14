use ash::vk;
use ash::version::DeviceV1_0;

use cgmath::{Deg, Matrix4, Point3, Vector3, Zero};

use std::rc::Rc;

use crate::resources::{Model, Resource};
use crate::common::RcCell;
use crate::{vec_remove_multiple, app};

pub mod transform;
pub use transform::*;
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
mod vk_vertex;
use vk_vertex::*;
mod buffers;
use buffers::*;
mod vk_mesh;
use vk_mesh::*;
mod descriptors;
use descriptors::*;

#[derive(Debug, Clone, Copy)]
pub struct DynamicRenderModelProperties {
    pub transform: Transform
}

pub struct DynamicRenderModel {
    model_resource: Resource<Model>,
    vk_meshes: Vec<VkMesh>,
    vk_textures: Vec<VkTexture>,
    properties: RcCell<DynamicRenderModelProperties>
}

impl DynamicRenderModel {
    pub fn is_active(&self) -> bool {
        self.properties.strong_count() > 1
    }

    pub fn draw(&self, cmd_buffer: &VkCmdBuffer) {
        for mesh in self.vk_meshes.iter() {
            mesh.draw_cmds(cmd_buffer);
        }
    }
}

#[repr(C)]
struct UBO {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>
}

impl Default for UBO {
    fn default() -> Self {
        UBO {
            model: Matrix4::zero(),
            view: Matrix4::zero(),
            proj: Matrix4::zero()
        }
    }
}

pub struct Renderer {
    vk_instance: VkInstance,
    physical_device: VkPhysicalDevice,
    device: Rc<VkLogicalDevice>,
    _graphics_queue: vk::Queue,
    _present_queue: vk::Queue,

    swapchain: Option<VkSwapchain>,
    render_pass: VkRenderPass,
    pipeline: VkPipeline,

    graphics_cmd_pool: Rc<VkCmdPool>,
    graphics_cmd_buffers: Vec<VkCmdBuffer>,

    descriptor_layout: VkDescriptorLayout,
    descriptor_pool: VkDescriptorPool,
    descriptor_sets: Vec<VkDescriptorSet>,

    dynamic_models: Vec<DynamicRenderModel>,
    ubo: Vec<VkUniformBuffer<UBO>>
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
        let descriptor_layout = VkDescriptorLayout::new(device.clone());
        let pipeline = VkPipeline::new(
            device.clone(),
            swapchain.get_extent(),
            &render_pass,
            &vec![&descriptor_layout],
            &vec![String::from("shader.vert"), String::from("shader.frag")]
        );
        swapchain.build_framebuffers(&render_pass);

        let graphics_cmd_pool = VkCmdPool::new(device.clone());
        let graphics_cmd_buffer = VkCmdBuffer::new(device.clone(), graphics_cmd_pool.clone(), swapchain.get_framebuffer_count() as u32);

        let mut ubo = Vec::new();
        for _ in 0..swapchain.get_framebuffer_count() {
            ubo.push(VkUniformBuffer::new(
                device.clone(),
                &physical_device
            ));
        }

        let mut descriptor_pool = VkDescriptorPool::new(device.clone());
        let descriptor_sets = VkDescriptorSet::new(
            device.clone(),
            &mut descriptor_pool,
            &descriptor_layout,
            &ubo
        );

        Box::new(Renderer {
            vk_instance: vk_instance,
            physical_device: physical_device,
            device: device,
            _graphics_queue: graphics_queue,
            _present_queue: present_queue,
            swapchain: Some(swapchain),
            render_pass: render_pass,
            pipeline: pipeline,
            graphics_cmd_pool: graphics_cmd_pool,
            graphics_cmd_buffers: graphics_cmd_buffer,
            descriptor_layout: descriptor_layout,
            descriptor_pool: descriptor_pool,
            descriptor_sets: descriptor_sets,

            dynamic_models: Vec::new(),
            ubo: ubo
        })
    }

    pub(crate) fn update(&mut self) {
        self.remove_unused_models();
        self.render();
    }

    fn remove_unused_models(&mut self) {
        let mut indices_to_remove = Vec::new();
        for (i, dynamic_model) in self.dynamic_models.iter().enumerate() {
            if !dynamic_model.is_active() {
                indices_to_remove.push(i);
            }
        }
        vec_remove_multiple(&mut self.dynamic_models, &mut indices_to_remove);
    }

    fn render(&mut self) {
        if let Some(swapchain) = self.swapchain.as_ref() {
            let (img_idx, fence) = swapchain.next_image();
            let img_available = swapchain.image_available_semaphore();
            let render_finished = swapchain.render_finished_semaphore();

            let time = app().time();
            let UBO = self.ubo[img_idx as usize].data();
            UBO.model = Matrix4::from_angle_y(Deg(90.0 * time));
            UBO.view = Matrix4::look_at(
                Point3::new(2.0, 0.0, 2.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            );
            UBO.proj = cgmath::perspective(
                Deg(60.0),
                app().window().width() as f32 / app().window().height() as f32,
                0.1,
                20.0,
            );

            let cmd_buffer = &self.graphics_cmd_buffers[img_idx as usize];
            cmd_buffer.reset();
            cmd_buffer.begin(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
            cmd_buffer.set_viewport(swapchain.get_extent());
            cmd_buffer.begin_render_pass(&self.render_pass, swapchain, img_idx as usize);
            cmd_buffer.bind_graphics_pipeline(&self.pipeline);
            cmd_buffer.bind_desc_set(&self.descriptor_sets[img_idx as usize], &self.pipeline);
            for dynamic_model in self.dynamic_models.iter() {
                dynamic_model.draw(cmd_buffer);
            }
            cmd_buffer.end_render_pass();
            cmd_buffer.end();

            cmd_buffer.submit(
                Some(&vec![img_available.as_ref()]),
                Some(&vec![render_finished.as_ref()]),
                Some(fence)
            );
            self.swapchain.as_mut().unwrap().present(img_idx, &vec![render_finished.as_ref()]);
        }
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
        if width > 0 && height > 0 {
            self.swapchain = Some(VkSwapchain::new(
                &self.vk_instance,
                self.device.clone(), &self.physical_device,
                width, height
            ));
            self.swapchain.as_mut().unwrap().build_framebuffers(&self.render_pass);
        }
    }

    pub fn create_dynamic_model(&mut self, model_resource: Resource<Model>) -> RcCell<DynamicRenderModelProperties> {
        let properties = RcCell::new(DynamicRenderModelProperties {
            transform: Transform::new()
        });

        let mut meshes = Vec::new();
        for mesh in model_resource.as_ref().meshes.iter() {
            meshes.push(VkMesh::new(
                self.device.clone(),
                &self.physical_device,
                self.graphics_cmd_pool.clone(),
                &mesh.vertices,
                &mesh.indices
            ));
        }

        let mut textures = Vec::new();
        for material in model_resource.as_ref().materials.iter() {
            textures.push(VkTexture::new(
                self.device.clone(),
                &self.physical_device,
                self.graphics_cmd_pool.clone(),
                material.as_ref().base_color_texture.clone()
            ));
        }

        let dynamic_render_model = DynamicRenderModel {
            model_resource: model_resource,
            vk_meshes: meshes,
            vk_textures: textures,
            properties: properties.clone()
        };
        self.dynamic_models.push(dynamic_render_model);

        properties
    }
}