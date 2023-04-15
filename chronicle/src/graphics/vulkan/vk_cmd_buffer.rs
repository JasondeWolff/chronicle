use std::rc::Rc;
use std::ptr;
use std::collections::HashMap;

use ash::vk;

use crate::graphics::*;

pub struct VkCmdBuffer {
    device: Rc<VkLogicalDevice>,
    cmd_pool: Rc<VkCmdPool>,
    cmd_buffer: vk::CommandBuffer,

    desc_pool: Rc<VkDescriptorPool>,
    desc_sets: HashMap<u32, VkDescriptorSet>,
    desc_layouts: HashMap<u32, Rc<VkDescriptorSetLayout>>,

    pipeline: Option<Rc<VkPipeline>>,

    tracked_buffers: Vec<Rc<VkBuffer>>
}

impl VkCmdBuffer {
    pub fn new(
        device: Rc<VkLogicalDevice>,
        cmd_pool: Rc<VkCmdPool>,
        desc_pool: Rc<VkDescriptorPool>
    ) -> Self {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: 1,
            command_pool: cmd_pool.get_cmd_pool(),
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffers = unsafe {
            device.get_device()
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers.")
        };

        VkCmdBuffer {
            device: device.clone(),
            cmd_pool: cmd_pool.clone(),
            cmd_buffer: command_buffers[0],
            desc_pool: desc_pool,
            desc_sets: HashMap::new(),
            desc_layouts: HashMap::new(),
            pipeline: None,
            tracked_buffers: Vec::new()
        }
    }

    pub fn get_cmd_buffer(&self) -> vk::CommandBuffer {
        self.cmd_buffer
    }

    pub fn reset(&mut self) {
        unsafe {
            self.pipeline = None;
            self.desc_sets.clear();
            self.tracked_buffers.clear();

            self.device.get_device()
                .reset_command_buffer(
                    self.cmd_buffer,
                    vk::CommandBufferResetFlags::empty()
                )
                .expect("Failed to execute queue reset.");
        }
    }

    pub fn begin(&self, flags: vk::CommandBufferUsageFlags) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: flags,
        };

        unsafe {
            self.device.get_device()
                .begin_command_buffer(self.cmd_buffer, &command_buffer_begin_info)
                .expect("Failed to begin Command Buffer.");
        }
    }

    pub fn end(&self) {
        unsafe {
            self.device.get_device()
                .end_command_buffer(self.cmd_buffer)
                .expect("Failed to end Command Buffer.");
        }
    }

    pub fn set_viewport(&self, extent: &vk::Extent2D) {
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: *extent,
        }];

        unsafe {
            self.device.get_device()
                .cmd_set_viewport(
                    self.cmd_buffer,
                    0,
                    &viewports
                );

            self.device.get_device()
                .cmd_set_scissor(
                    self.cmd_buffer, 
                    0, 
                    &scissors
                );
        }
    }

    pub fn begin_render_pass(&self, render_pass: &VkRenderPass, swapchain: &VkSwapchain, frame_idx: usize) {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: render_pass.get_render_pass(),
            framebuffer: *swapchain.get_framebuffer(frame_idx),
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: *swapchain.get_extent(),
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        unsafe {
            self.device.get_device().cmd_begin_render_pass(
                self.cmd_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    pub fn end_render_pass(&self) {
        unsafe {
            self.device.get_device()
                .cmd_end_render_pass(self.cmd_buffer);
        }
    }

    pub fn bind_graphics_pipeline(&mut self, pipeline: Rc<VkPipeline>) {
        self.pipeline = Some(pipeline.clone());

        unsafe {
            self.device.get_device()
                .cmd_bind_pipeline(
                    self.cmd_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline.get_pipeline(),
                );
        }
    }

    pub fn bind_vertex_buffer(&self, vertex_buffer: &VkVertexBuffer) {
        unsafe {
            self.device.get_device()
                .cmd_bind_vertex_buffers(
                    self.cmd_buffer,
                    0,
                    &[vertex_buffer.get_buffer()],
                    &[0_u64]
                );
        }
    }

    pub fn bind_index_buffer(&self, index_buffer: &VkIndexBuffer) {
        unsafe {
            self.device.get_device()
                .cmd_bind_index_buffer(
                    self.cmd_buffer,
                    index_buffer.get_buffer(),
                    0,
                    vk::IndexType::UINT32
                );
        }
    }

    pub fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        unsafe {
            self.device.get_device()
                .cmd_draw(
                    self.cmd_buffer,
                    vertex_count,
                    instance_count,
                    first_vertex,
                    first_instance
                );
        }
    }

    pub fn draw_indexed(&self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: u32, first_instance: u32) {
        unsafe {
            self.device.get_device()
                .cmd_draw_indexed(
                    self.cmd_buffer,
                    index_count,
                    instance_count,
                    first_index,
                    vertex_offset as i32,
                    first_instance
                );
        }
    }

    pub fn copy_buffers(&self, src_buffer: &VkBuffer, dst_buffer: &VkBuffer) {
        assert_eq!(src_buffer.get_size(), dst_buffer.get_size(), "Failed to copy buffers.");
        
        let copy_regions = [vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: src_buffer.get_size(),
        }];

        unsafe {
            self.device.get_device()
                .cmd_copy_buffer(
                    self.cmd_buffer,
                    src_buffer.get_buffer(),
                    dst_buffer.get_buffer(),
                    &copy_regions
                );
        }
    }

    pub fn copy_buffer_to_image(&self, src_buffer: &VkBuffer, dst_image: &VkImage) {
        let buffer_image_regions = [vk::BufferImageCopy {
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_extent: vk::Extent3D {
                width: dst_image.width(),
                height: dst_image.height(),
                depth: 1,
            },
            buffer_offset: 0,
            buffer_image_height: 0,
            buffer_row_length: 0,
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
        }];

        unsafe {
            self.device.get_device()
                .cmd_copy_buffer_to_image(
                    self.cmd_buffer,
                    src_buffer.get_buffer(),
                    dst_image.get_image(),
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &buffer_image_regions,
                );
        }
    }

    pub fn transition_image_layout(&self,
        image: &VkImage,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout
    ) {
        let src_access_mask;
        let dst_access_mask;
        let source_stage;
        let destination_stage;

        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::TRANSFER;
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            dst_access_mask = vk::AccessFlags::SHADER_READ;
            source_stage = vk::PipelineStageFlags::TRANSFER;
            destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        } else {
            panic!("Unsupported layout transition!")
        }

        let image_barriers = [vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: image.get_image(),
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        }];

        unsafe {
            self.device.get_device()
                .cmd_pipeline_barrier(
                    self.cmd_buffer,
                    source_stage,
                    destination_stage,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &image_barriers,
                );
        }
    }

    pub fn set_desc_layout(&mut self,
        set: u32,
        layout: Rc<VkDescriptorSetLayout>
    ) {
        self.desc_layouts.insert(set, layout);
    }

    pub fn set_desc_buffer<T: Default>(&mut self,
        set: u32,
        binding: u32,
        desc_type: vk::DescriptorType,
        uniform_buffer: VkUniformBuffer<T>
    ) {
        let desc_layout = self.desc_layouts.get(&set)
                                                                    .expect("Failed to set desc buffer. (Missing desc layout_");

        let desc_set = match self.desc_sets.get(&set) {
            Some(desc_set) => desc_set,
            None => {
                let desc_set = VkDescriptorSet::new(
                    self.device.clone(),
                    self.desc_pool.clone(),
                    desc_layout.clone()
                );
                self.desc_sets.insert(set, desc_set);
                self.desc_sets.get(&set).as_ref().unwrap()
            }
        };

        let descriptor_buffer_info = [vk::DescriptorBufferInfo {
            buffer: uniform_buffer.get_buffer(),
            offset: 0,
            range: std::mem::size_of::<T>() as u64,
        }];

        let descriptor_write_sets = [
            vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: std::ptr::null(),
                dst_set: desc_set.get_desc_set(),
                dst_binding: binding,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: desc_type,
                p_image_info: std::ptr::null(),
                p_buffer_info: descriptor_buffer_info.as_ptr(),
                p_texel_buffer_view: std::ptr::null(),
            }
        ];

        unsafe {
            self.device.get_device()
                .update_descriptor_sets(&descriptor_write_sets, &[]);
        }

        self.tracked_buffers.push(uniform_buffer.track_buffer());
    }

    pub fn bind_desc_sets(&self) {
        let mut sorted_desc_sets: Vec<_> = self.desc_sets.iter().collect();
        sorted_desc_sets.sort_by(|x, y| x.0.cmp(&y.0));

        let mut desc_set_ptrs = Vec::new();
        for desc_set in &sorted_desc_sets {
            desc_set_ptrs.push(desc_set.1.get_desc_set());
        }

        unsafe {
            self.device.get_device()
                .cmd_bind_descriptor_sets(
                    self.cmd_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline.as_ref().unwrap().get_layout(),
                    0,
                    &desc_set_ptrs,
                    &[]
                );
        }
    }
}

impl Drop for VkCmdBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .free_command_buffers(
                    self.cmd_pool.get_cmd_pool(),
                    &[self.cmd_buffer]
                );
        }
    }
}