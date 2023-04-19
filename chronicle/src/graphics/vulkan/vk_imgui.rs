extern crate imgui;

use ash::vk;
use cgmath::Vector4;
use memoffset::offset_of;

use crate::graphics::*;
use crate::resources::Texture;

pub struct VkImGui {
    context: imgui::Context,
    renderer: Renderer
}

impl VkImGui {
    pub fn new(
        app: RcCell<VkApp>,
        render_pass: &VkRenderPass
    ) -> Self {
        let mut context = imgui::Context::create();
        let renderer = Renderer::new(
            app.clone(),
            render_pass,
            &mut context
        );

        let app = app.as_ref();
        let swapchain = app.swapchain.as_ref().unwrap().as_ref();
        let extent = swapchain.get_extent();
        context.io_mut().display_size[0] = extent.width as f32;
        context.io_mut().display_size[1] = extent.height as f32;

        VkImGui {
            context: context,
            renderer: renderer
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.context.io_mut().display_size[0] = width as f32;
        self.context.io_mut().display_size[1] = height as f32;
    }

    pub fn mouse_button_event(&mut self, button: imgui::MouseButton, down: bool) {
        self.context.io_mut().add_mouse_button_event(button, down);
    }

    pub fn mouse_pos_event(&mut self, x: f32, y: f32) {
        self.context.io_mut().add_mouse_pos_event([x, y]);
    }

    pub fn new_frame(&mut self) -> &mut imgui::Ui {
        self.context.new_frame()
    }

    pub fn render(&mut self, app: &mut VkApp, render_pass: Rc<VkRenderPass>) -> (Rc<VkFence>, Rc<VkSemaphore>) {
        self.renderer.render(app, render_pass, &mut self.context)
    }
}

struct PushConstant {
    proj: Matrix4<f32>
}

struct ImGuiVert {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: [f32; 4]
}

impl VkVertexDescs for ImGuiVert {
    fn get_binding_desc() -> Vec<vk::VertexInputBindingDescription> {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }].to_vec()
    }

    fn get_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Self, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Self, uv) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Self, col) as u32,
            }
        ].to_vec()
    }
}

struct Renderer {
    device: Rc<VkLogicalDevice>,
    pipeline: Rc<VkPipeline>,
    desc_layout: Rc<VkDescriptorSetLayout>,
    texture: VkTexture,
    sampler: VkSampler
}

impl Renderer {
    fn new(
        app: RcCell<VkApp>,
        render_pass: &VkRenderPass,
        imgui: &mut imgui::Context
    ) -> Self {
        let (pipeline, desc_layout) = Self::create_pipeline(app.clone(), render_pass);
        let device = app.as_ref().get_device();
        let swapchain = app.as_ref().get_swapchain().unwrap();

        let font_atlas = imgui.fonts();
        let atlas_texture = font_atlas.build_rgba32_texture();
        let texture = Resource::new(Texture {
            data: atlas_texture.data.to_vec(),
            width: atlas_texture.width,
            height: atlas_texture.height,
            channel_count: 4,
            mip_levels: 1 // Mip mapping???
        }, "tmp".to_owned());
        let texture = VkTexture::new(app.clone(), texture);
        let sampler = VkSampler::new(device.clone(), &texture);

        Renderer {
            device: device,
            pipeline: pipeline,
            desc_layout: desc_layout,
            texture: texture,
            sampler: sampler
        }
    }

    fn create_pipeline(
        app: RcCell<VkApp>,
        render_pass: &VkRenderPass
    ) -> (Rc<VkPipeline>, Rc<VkDescriptorSetLayout>) {
        let app = app.as_mut();
        let device = app.get_device();
        let swapchain_extent = *app.get_swapchain().unwrap().as_ref().get_extent();

        let descriptor_layout = VkDescriptorSetLayout::new(device.clone(), &vec![
            vk::DescriptorSetLayoutBinding {
                binding: 0,
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
                size: std::mem::size_of::<PushConstant>() as u32
            }
        ];

        let pipeline = VkPipeline::new::<ImGuiVert>(
            device.clone(),
            &swapchain_extent,
            &render_pass,
            &vec![&descriptor_layout],
            &push_constants,
            &vec![String::from("imgui.vert"), String::from("imgui.frag")],
            vk::CullModeFlags::NONE,
            vk::FALSE
        );

        (pipeline, descriptor_layout)
    }

    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Matrix4<f32> {
        let rml = right - left;
        let rpl = right + left;
        let tmb = top - bottom;
        let tpb = top + bottom;
        let fmn = far - near;
        Matrix4 {
            x: Vector4::new(2.0 / rml, 0.0, 0.0, 0.0),
            y: Vector4::new(0.0, -2.0 / tmb, 0.0, 0.0),
            z: Vector4::new(0.0, 0.0, -1.0 / fmn, 0.0),
            w: Vector4::new(-(rpl / rml), -(tpb / tmb), -(near / fmn), 1.0)
        }
    }

    pub fn render(&mut self, app: &mut VkApp, render_pass: Rc<VkRenderPass>, ctx: &mut imgui::Context) -> (Rc<VkFence>, Rc<VkSemaphore>) {
        use imgui::{DrawVert, DrawIdx, DrawCmd, DrawCmdParams};

        let [width, height] = ctx.io().display_size;
        let [scale_w, scale_h] = ctx.io().display_framebuffer_scale;
        let fb_width = width * scale_w;
        let fb_height = height * scale_h;

        let proj_matrix = Self::ortho(
            0.0,
            width,
            0.0,
            -height,
            -1.0,
            1.0,
        );

        let draw_data = ctx.render();

        let mut vertex_buffers = Vec::new();
        let mut index_buffers = Vec::new();
        for draw_list in draw_data.draw_lists() {
            let vtx_buffer: Vec<ImGuiVert> = draw_list.vtx_buffer().into_iter().map(|x| -> ImGuiVert {
                ImGuiVert {
                    pos: x.pos.clone(),
                    uv: x.uv.clone(),
                    col: [x.col[0] as f32 / 255.0, x.col[1] as f32 / 255.0, x.col[2] as f32 / 255.0, x.col[3] as f32 / 255.0]
                }
            }).collect();
            let idx_buffer: Vec<u32> = draw_list.idx_buffer().into_iter().map(|x| -> u32 { *x as u32 }).collect();

            vertex_buffers.push(VkVertexBuffer::new(
                app,
                &vtx_buffer,
                false
            ));
            index_buffers.push(VkIndexBuffer::new(
                app,
                &idx_buffer,
                false
            ));
        }

        let swapchain = app.get_swapchain();
        if let Some(swapchain) = app.get_swapchain() {
            let cmd_queue = app.get_cmd_queue();
            let cmd_buffer = cmd_queue.get_cmd_buffer(); {
                let mut cmd_buffer = cmd_buffer.as_mut();
                cmd_buffer.reset();
                cmd_buffer.begin(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

                let swapchain = swapchain.as_ref();
                cmd_buffer.set_viewport(&vk::Extent2D {
                    width: fb_width as u32,
                    height: fb_height as u32
                });
                cmd_buffer.begin_render_pass(&render_pass, &swapchain, false);

                cmd_buffer.bind_graphics_pipeline(self.pipeline.clone());
                cmd_buffer.set_desc_layout(0, self.desc_layout.clone());
                cmd_buffer.set_desc_sampler(0, 0, vk::DescriptorType::COMBINED_IMAGE_SAMPLER, &self.sampler, &mut self.texture);
                cmd_buffer.bind_desc_sets();
        
                cmd_buffer.push_constant(
                    &PushConstant {
                        proj: proj_matrix
                    },
                    vk::ShaderStageFlags::VERTEX
                );
        
                let draw_data = ctx.render();
                for (i, draw_list) in draw_data.draw_lists().into_iter().enumerate() {        
                    let vertex_buffer = &vertex_buffers[i];
                    let index_buffer = &index_buffers[i];
                    cmd_buffer.bind_vertex_buffer(&vertex_buffer);
                    cmd_buffer.bind_index_buffer(&index_buffer);
        
                    for cmd in draw_list.commands() {
                        match cmd {
                                DrawCmd::Elements {
                                    count,
                                    cmd_params: DrawCmdParams {
                                      clip_rect: [x, y, z, w],
                                      texture_id,
                                      idx_offset,
                                      ..
                                    },
                                } => {
                                    // cmd_buffer.set_scissor(vk::Rect2D {
                                    //     offset: vk::Offset2D {
                                    //         x: (x * scale_w) as i32,
                                    //         y: (fb_height - w * scale_h) as i32
                                    //     },
                                    //     extent: vk::Extent2D {
                                    //         width: ((z - x) * scale_w) as u32,
                                    //         height: ((w - y) * scale_h) as u32
                                    //     }
                                    // });
            
                                    cmd_buffer.draw_indexed(
                                        count as u32,
                                        1,
                                        idx_offset as u32,
                                        0,
                                        0
                                    );
                                }
                                _ => {}
                            }
                    }
                }

                cmd_buffer.end_render_pass();
                cmd_buffer.end();
            }

            let mut swapchain = swapchain.as_mut();
            let render_finished = swapchain.render_finished_semaphore();

            let fence = cmd_queue.submit_cmd_buffer(
                cmd_buffer,
                None,
                Some(&vec![render_finished.as_ref()])
            );

            return (fence, render_finished);
        }

        panic!("NOOT");
    }
}