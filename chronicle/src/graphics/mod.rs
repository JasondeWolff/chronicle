use std::rc::Rc;

use ash::vk;

use cgmath::{Deg, Matrix4, Point3, Vector3};

use crate::resources::{Model, Resource};
pub(crate) use crate::common::RcCell;
use crate::vec_remove_multiple;

pub mod transform;
pub use transform::*;
pub mod window;
pub use window::*;
mod vulkan;
use vulkan::*;


// TODO:
// [X] MSAA
// [X] Push constants
// [ ] Camera struct & Input handling
// [ ] multiple objects
// [ ] materials
// [ ] imgui

#[derive(Debug, Clone, Copy)]
pub struct DynamicRenderModelProperties {
    pub transform: Transform
}

pub struct DynamicRenderModel {
    _model_resource: Resource<Model>,
    vk_meshes: Vec<VkMesh>,
    vk_textures: Vec<VkTexture>,
    vk_samplers: Vec<VkSampler>,
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

// #[repr(C)]
// struct UBO {
//     model: Matrix4<f32>,
//     view: Matrix4<f32>,
//     proj: Matrix4<f32>
// }

// impl ToAny for UBO {
//     fn as_any(&mut self) -> &mut dyn std::any::Any {
//         self
//     }
// }

#[repr(C)]
struct MVP {
    mvp: Matrix4<f32>
}

pub struct Renderer {
    app: RcCell<VkApp>,

    render_img: RcCell<VkImage>,
    render_pass: Rc<VkRenderPass>,
    pipeline: Rc<VkPipeline>,

    descriptor_layout: Rc<VkDescriptorSetLayout>,

    dynamic_models: Vec<DynamicRenderModel>
}

impl Renderer {
    pub(crate) fn init(window: &Window) -> Box<Self> {
        let app = VkApp::new(window);
        let device = app.get_device();
        let physical_device = app.get_physical_device();
        let swapchain = app.get_swapchain().unwrap();
        let mut swapchain = swapchain.as_mut();

        let max_sample_count = physical_device.get_max_sample_count();
        let render_img = RcCell::new(VkImage::new(
            device.clone(),
            swapchain.get_extent().width, swapchain.get_extent().height,
            1,
            swapchain.get_color_format(),
            max_sample_count,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            physical_device.get_mem_properties()
        ));

        let render_pass = VkRenderPass::new(
            device.clone(),
            swapchain.get_color_format(),
            swapchain.get_depth_format(),
            max_sample_count
        );

        swapchain.build_framebuffers(
            physical_device,
            render_pass.clone(),
            render_img.clone()
        );

        let descriptor_layout = VkDescriptorSetLayout::new(device.clone(), &vec![
            // vk::DescriptorSetLayoutBinding {
            //     binding: 0,
            //     descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            //     descriptor_count: 1,
            //     stage_flags: vk::ShaderStageFlags::VERTEX,
            //     p_immutable_samplers: std::ptr::null(),
            // },
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                p_immutable_samplers: std::ptr::null(),
            },
        ]);
        
        let push_constants = vec![
            vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::VERTEX,
                offset: 0,
                size: std::mem::size_of::<MVP>() as u32
            }
        ];

        let pipeline = VkPipeline::new(
            device.clone(),
            swapchain.get_extent(),
            &render_pass,
            &vec![&descriptor_layout],
            &push_constants,
            &vec![String::from("shader.vert"), String::from("shader.frag")]
        );

        Box::new(Renderer {
            app: RcCell::new(app),
            render_img: render_img,
            render_pass: render_pass,
            pipeline: pipeline,
            descriptor_layout: descriptor_layout,

            dynamic_models: Vec::new()
        })
    }

    pub(crate) fn update(&mut self) {
        self.app.as_mut().update();

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
        let mut app = self.app.as_mut();

        if let Some(swapchain) = app.get_swapchain() {
            {
                let mut swapchain = swapchain.as_mut();
                swapchain.next_image();
            }
            
            // let uniform_buffer = app.uniform_buffer::<UBO>("matrices"); {
            //     let mut uniform_buffer = uniform_buffer.as_mut();
            //     let ubo: &mut UBO = uniform_buffer.data();
            //     ubo.model = Matrix4::from_angle_y(Deg(45.0 * crate::app().time()));
            //     ubo.view = Matrix4::look_at(
            //         Point3::new(2.0, 0.0, 2.0),
            //         Point3::new(0.0, 0.0, 0.0),
            //         Vector3::new(0.0, 1.0, 0.0),
            //     );
            //     ubo.proj = cgmath::perspective(
            //         Deg(60.0),
            //         crate::app().window().width() as f32 / crate::app().window().height() as f32,
            //         0.1,
            //         20.0,
            //     );
            // }

            let view_matrix = Matrix4::look_at(
                Point3::new(2.0, 0.0, 2.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            );
            let proj_matrix = cgmath::perspective(
                Deg(60.0),
                crate::app().window().width() as f32 / crate::app().window().height() as f32,
                0.1,
                20.0,
            );

            let cmd_queue = app.get_cmd_queue();
            let cmd_buffer = cmd_queue.get_cmd_buffer(); {
                let mut cmd_buffer = cmd_buffer.as_mut();
                cmd_buffer.reset();
                cmd_buffer.begin(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

                let swapchain = swapchain.as_ref();
                cmd_buffer.set_viewport(swapchain.get_extent());
                cmd_buffer.begin_render_pass(&self.render_pass, &swapchain);
                
                cmd_buffer.bind_graphics_pipeline(self.pipeline.clone());

                cmd_buffer.set_desc_layout(0, self.descriptor_layout.clone());
                //cmd_buffer.set_desc_buffer(0, 0, vk::DescriptorType::UNIFORM_BUFFER, uniform_buffer.clone());
                let dynamic_model = &mut self.dynamic_models[0];
                cmd_buffer.set_desc_sampler(0, 1, vk::DescriptorType::COMBINED_IMAGE_SAMPLER, &dynamic_model.vk_samplers[0], &mut dynamic_model.vk_textures[0]);
                cmd_buffer.bind_desc_sets();
                
                let model_matrix = Matrix4::from_angle_y(Deg(45.0 * crate::app().time()));
                cmd_buffer.push_constant(
                    &MVP {
                        mvp: proj_matrix * view_matrix * model_matrix
                    },
                    vk::ShaderStageFlags::VERTEX
                );

                for dynamic_model in self.dynamic_models.iter() {
                    

                    dynamic_model.draw(&cmd_buffer);
                }
                cmd_buffer.end_render_pass();
                cmd_buffer.end();
            }

            let mut swapchain = swapchain.as_mut();
            let img_available = swapchain.image_available_semaphore();
            let render_finished = swapchain.render_finished_semaphore();

            let fence = cmd_queue.submit_cmd_buffer(
                cmd_buffer,
                Some(&vec![img_available.as_ref()]),
                Some(&vec![render_finished.as_ref()])
            );

            swapchain.present(fence.clone(), &vec![render_finished.as_ref()]);
        }
    }

    pub(crate) fn wait_idle(&self) {
        let device = self.app.as_ref().get_device();
        device.wait_idle();
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        let mut app = self.app.as_mut();
        app.resize(width, height);

        if let Some(swapchain) = app.get_swapchain() {
            let mut swapchain = swapchain.as_mut();
            let device = app.get_device();
            let physical_device = app.get_physical_device();

            let sample_count = self.render_img.as_ref().sample_count();

            self.render_img = RcCell::new(VkImage::new(
                device.clone(),
                swapchain.get_extent().width, swapchain.get_extent().height,
                1,
                swapchain.get_color_format(),
                sample_count,
                vk::ImageTiling::OPTIMAL,
                vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                physical_device.get_mem_properties()
            ));

            swapchain.build_framebuffers(
                physical_device,
                self.render_pass.clone(),
                self.render_img.clone()
            );
        }
    }

    pub fn create_dynamic_model(&mut self, model_resource: Resource<Model>) -> RcCell<DynamicRenderModelProperties> {
        let properties = RcCell::new(DynamicRenderModelProperties {
            transform: Transform::new()
        });

        let mut meshes = Vec::new();
        for mesh in model_resource.as_ref().meshes.iter() {
            meshes.push(VkMesh::new(
                self.app.clone(),
                &mesh.vertices,
                &mesh.indices
            ));
        }

        let mut textures = Vec::new();
        for material in model_resource.as_ref().materials.iter() {
            textures.push(VkTexture::new(
                self.app.clone(),
                material.as_ref().base_color_texture.clone()
            ));
        }

        let samplers = vec![VkSampler::new( // HARDCODED!!!!
            self.app.as_ref().get_device(),
            &textures[0]
        )];

        let dynamic_render_model = DynamicRenderModel {
            _model_resource: model_resource,
            vk_meshes: meshes,
            vk_textures: textures,
            vk_samplers: samplers,
            properties: properties.clone()
        };
        self.dynamic_models.push(dynamic_render_model);

        properties
    }
}