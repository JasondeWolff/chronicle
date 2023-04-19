pub mod mod_structs;
pub use mod_structs::*;

pub mod transform;
pub use transform::*;
pub mod camera;
pub use camera::*;

pub type ImGuiUI = imgui::Ui;

mod vulkan;
use vulkan::*;

use std::collections::HashMap;
use std::rc::Rc;
use ash::vk;
use cgmath::{Deg, Matrix4};

use crate::Window;
use crate::resources::{Model, Resource, Mesh, Texture, model};
use crate::common::{RcCell, vec_remove_multiple};

// TODO
// [ ] ImGui resizing
// [ ] Debug jittering artifacts
// [ ] gpu-memory allocator

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
    imgui: VkImGui,

    render_img: RcCell<VkImage>,
    render_pass: Rc<VkRenderPass>,
    present_render_pass: Rc<VkRenderPass>,
    pipeline: Rc<VkPipeline>,

    descriptor_layout: Rc<VkDescriptorSetLayout>,

    models: HashMap<Resource<Model>, Vec<VkMesh>>,
    textures: HashMap<Resource<Texture>, VkTexture>,
    samplers: HashMap<u32, VkSampler>,

    cameras: Vec<RenderCamera>,
    dynamic_models: Vec<DynamicRenderModel>
}

impl Renderer {
    pub(crate) fn init(window: &Window) -> Box<Self> {
        let app = VkApp::new(window);
        let device = app.get_device();
        let physical_device = app.get_physical_device();
        let swapchain = app.get_swapchain().unwrap();

        let render_img;
        let render_pass;
        let present_render_pass;
        let descriptor_layout;
        let pipeline;
        {
            let mut swapchain = swapchain.as_mut();

            let max_sample_count = physical_device.get_max_sample_count();
            render_img = RcCell::new(VkImage::new(
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

            render_pass = VkRenderPass::new(
                device.clone(),
                swapchain.get_color_format(),
                swapchain.get_depth_format(),
                max_sample_count,
                vk::AttachmentLoadOp::CLEAR,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            );
            present_render_pass = VkRenderPass::new(
                device.clone(),
                swapchain.get_color_format(),
                swapchain.get_depth_format(),
                max_sample_count,
                vk::AttachmentLoadOp::LOAD,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::PRESENT_SRC_KHR
            );

            swapchain.build_framebuffers(
                physical_device,
                present_render_pass.clone(),
                render_img.clone()
            );

            descriptor_layout = VkDescriptorSetLayout::new(device.clone(), &vec![
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
                }
            ]);
            
            let push_constants = vec![
                vk::PushConstantRange {
                    stage_flags: vk::ShaderStageFlags::VERTEX,
                    offset: 0,
                    size: std::mem::size_of::<MVP>() as u32
                }
            ];

            pipeline = VkPipeline::new::<VkVertex>(
                device.clone(),
                swapchain.get_extent(),
                &render_pass,
                &vec![&descriptor_layout],
                &push_constants,
                &vec![String::from("shader.vert"), String::from("shader.frag")],
                vk::CullModeFlags::BACK,
                vk::TRUE
            );
        }

        let app = RcCell::new(app);
        let imgui = VkImGui::new(
            app.clone(),
            &render_pass
        );

        Box::new(Renderer {
            app: app,
            imgui: imgui,
            render_img: render_img,
            render_pass: render_pass,
            present_render_pass: present_render_pass,
            pipeline: pipeline,
            descriptor_layout: descriptor_layout,

            models: HashMap::new(),
            textures: HashMap::new(),
            samplers: HashMap::new(),

            cameras: Vec::new(),
            dynamic_models: Vec::new()
        })
    }

    pub(crate) fn update(&mut self) {
        self.app.as_mut().update();

        self.remove_unused_resources();
        self.render();
    }

    pub(crate) fn imgui_frame(&mut self) -> &mut ImGuiUI {
        self.imgui.new_frame()
    }

    pub(crate) fn imgui(&mut self) -> &mut VkImGui {
        &mut self.imgui
    }

    fn remove_unused_resources(&mut self) {
        { // Cameras
            let mut indices_to_remove = Vec::new();
            for (i, camera) in self.cameras.iter().enumerate() {
                if !camera.is_active() {
                    indices_to_remove.push(i);
                }
            }
            vec_remove_multiple(&mut self.dynamic_models, &mut indices_to_remove);
        }
        { // Dynamic models
            let mut indices_to_remove = Vec::new();
            for (i, dynamic_model) in self.dynamic_models.iter().enumerate() {
                if !dynamic_model.is_active() {
                    indices_to_remove.push(i);
                }
            }
            vec_remove_multiple(&mut self.dynamic_models, &mut indices_to_remove);
        }
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

            let mut main_camera = None;
            for camera in &self.cameras {
                if camera.properties.as_ref().main {
                    main_camera = Some(camera.properties.clone());
                }
            }
            let main_camera = &mut main_camera.as_ref().expect("Failed to find main camera.").as_mut().camera;
            let view_matrix = *main_camera.get_view_matrix();
            let proj_matrix = *main_camera.get_proj_matrix();

            {
                let cmd_queue = app.get_cmd_queue();
                let cmd_buffer = cmd_queue.get_cmd_buffer(); {
                    let mut cmd_buffer = cmd_buffer.as_mut();
                    cmd_buffer.reset();
                    cmd_buffer.begin(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

                    let swapchain = swapchain.as_ref();
                    cmd_buffer.set_viewport(swapchain.get_extent());
                    cmd_buffer.begin_render_pass(&self.render_pass, &swapchain, true);

                    cmd_buffer.bind_graphics_pipeline(self.pipeline.clone());

                    for dynamic_model in self.dynamic_models.iter() {
                        let mut model_properties = dynamic_model.properties.as_mut();
                        let model_matrix = model_properties.transform.get_matrix(false);
                        cmd_buffer.push_constant(
                            &MVP {
                                mvp: proj_matrix * view_matrix * model_matrix
                            },
                            vk::ShaderStageFlags::VERTEX
                        );

                        let vk_meshes = self.models.get(&dynamic_model.model_resource).unwrap();
                        for (i, mesh) in dynamic_model.model_resource.as_ref().meshes.iter().enumerate() {
                            let material = dynamic_model.model_resource.as_ref().materials[mesh.material_idx].clone();
                            let material = material.as_ref();

                            cmd_buffer.set_desc_layout(0, self.descriptor_layout.clone());

                            let base_color_texture = self.textures.get_mut(&material.base_color_texture);
                            if let Some(texture) = base_color_texture {
                                let sampler = self.samplers.get(&texture.mip_levels()).unwrap();
                                cmd_buffer.set_desc_sampler(0, 1, vk::DescriptorType::COMBINED_IMAGE_SAMPLER, sampler, texture);
                            }

                            cmd_buffer.bind_desc_sets();

                            vk_meshes[i].draw_cmds(&mut cmd_buffer);
                        }
                    }

                    cmd_buffer.end_render_pass();
                    cmd_buffer.end();
                }

                let swapchain = swapchain.as_mut();
                let img_available = swapchain.image_available_semaphore();

                let _ = cmd_queue.submit_cmd_buffer(
                    cmd_buffer,
                    Some(&vec![&img_available]),
                    None
                );
            }

            let (fence, render_finished) = self.imgui.render(&mut app, self.present_render_pass.clone());

            swapchain.as_mut().present(fence.clone(), &vec![render_finished.as_ref()]);
        }
    }

    pub(crate) fn wait_idle(&self) {
        let device = self.app.as_ref().get_device();
        device.wait_idle();
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        let mut app = self.app.as_mut();
        app.resize(width, height);
        self.imgui.resize(width, height);

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

    pub fn create_camera(&mut self) -> RcCell<RenderCameraProperties> {
        let properties = RcCell::new(RenderCameraProperties {
            camera: Camera::new(),
            main: false
        });

        self.cameras.push(RenderCamera {
            properties: properties.clone()
        });

        properties
    }

    fn store_texture(&mut self, texture_resource: Resource<Texture>) {
        if self.textures.get(&texture_resource).is_none() {
            let texture = VkTexture::new(
                self.app.clone(),
                texture_resource.clone()
            );

            if self.samplers.get(&texture.mip_levels()).is_none() {
                self.samplers.insert(
                    texture.mip_levels(),
                    VkSampler::new(
                        self.app.as_ref().get_device(),
                        &texture
                    )
                );
            }

            self.textures.insert(
                texture_resource,
                texture
            );
        }
    }

    pub fn create_dynamic_model(&mut self, model_resource: Resource<Model>) -> RcCell<DynamicRenderModelProperties> {
        let properties = RcCell::new(DynamicRenderModelProperties {
            transform: Transform::new()
        });

        if self.models.get(&model_resource).is_none() {
            let mut meshes = Vec::new();
            for mesh in model_resource.as_ref().meshes.iter() {
                meshes.push(VkMesh::new(
                    &mut self.app.as_mut(),
                    &mesh.vertices,
                    &mesh.indices
                ));
            }
            self.models.insert(model_resource.clone(), meshes);
        }

        for material in &model_resource.as_ref().materials {
            self.store_texture(material.as_ref().base_color_texture.clone());
            self.store_texture(material.as_ref().normal_texture.clone());
            self.store_texture(material.as_ref().metallic_roughness_texture.clone());
            self.store_texture(material.as_ref().occlusion_texture.clone());
            self.store_texture(material.as_ref().emissive_texture.clone());
        }

        let dynamic_render_model = DynamicRenderModel {
            model_resource: model_resource,
            properties: properties.clone()
        };
        self.dynamic_models.push(dynamic_render_model);

        properties
    }
}