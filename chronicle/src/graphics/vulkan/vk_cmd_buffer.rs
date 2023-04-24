use std::ptr;
use std::collections::HashMap;
use std::cmp::max;

use ash::vk;

use crate::graphics::*;

pub struct VkCmdBuffer {
    device: Arc<VkLogicalDevice>,
    allocator: ArcMutex<Allocator>,

    cmd_pool: Arc<VkCmdPool>,
    cmd_buffer: vk::CommandBuffer,

    desc_pool: Arc<VkDescriptorPool>,
    desc_sets: HashMap<u32, Arc<VkDescriptorSet>>,
    desc_layouts: HashMap<u32, Arc<VkDescriptorSetLayout>>,

    pipeline: Option<Arc<VkPipeline>>,

    tracked_buffers: Vec<Arc<VkBuffer>>,
    tracked_desc_sets: Vec<Arc<VkDescriptorSet>>,
}

impl VkCmdBuffer {
    pub fn new(
        device: Arc<VkLogicalDevice>,
        allocator: ArcMutex<Allocator>,
        cmd_pool: Arc<VkCmdPool>,
        desc_pool: Arc<VkDescriptorPool>
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
            allocator: allocator.clone(),
            cmd_pool: cmd_pool.clone(),
            cmd_buffer: command_buffers[0],
            desc_pool: desc_pool,
            desc_sets: HashMap::new(),
            desc_layouts: HashMap::new(),
            pipeline: None,
            tracked_buffers: Vec::new(),
            tracked_desc_sets: Vec::new()
        }
    }

    pub fn get_cmd_buffer(&self) -> vk::CommandBuffer {
        self.cmd_buffer
    }

    pub fn reset(&mut self) {
        unsafe {
            self.pipeline = None;
            self.desc_layouts.clear();
            self.tracked_buffers.clear();
            self.tracked_desc_sets.clear();

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

    pub fn set_scissor(&self, scissor: vk::Rect2D) {
        unsafe {
            self.device.get_device()
                .cmd_set_scissor(
                    self.cmd_buffer, 
                    0, 
                    &[scissor]
                );
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

    pub fn begin_render_pass(&self, render_pass: &VkRenderPass, swapchain: &VkSwapchain) {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            }
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: render_pass.get_render_pass(),
            framebuffer: *swapchain.get_current_framebuffer(),
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

    pub fn bind_graphics_pipeline(&mut self, pipeline: Arc<VkPipeline>) {
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

    pub fn bind_vertex_buffer<T>(&mut self, vertex_buffer: &VkDataBuffer<T>) {
        unsafe {
            self.device.get_device()
                .cmd_bind_vertex_buffers(
                    self.cmd_buffer,
                    0,
                    &[vertex_buffer.get_buffer().get_buffer()],
                    &[0_u64]
                );
        }

        self.tracked_buffers.push(vertex_buffer.get_buffer());
    }

    pub fn bind_index_buffer(&mut self, index_buffer: &VkDataBuffer<u32>) {
        unsafe {
            self.device.get_device()
                .cmd_bind_index_buffer(
                    self.cmd_buffer,
                    index_buffer.get_buffer().get_buffer(),
                    0,
                    vk::IndexType::UINT32
                );
        }

        self.tracked_buffers.push(index_buffer.get_buffer());
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
        new_layout: vk::ImageLayout,
        mip_levels: u32
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
        } else if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask =
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
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
                level_count: mip_levels,
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

    pub fn generate_mips(&self, image: &VkImage, mip_levels: u32) {
        let mut image_barrier = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::empty(),
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::UNDEFINED,
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
        };

        let mut mip_width = image.width() as i32;
        let mut mip_height = image.height() as i32;

        for i in 1..mip_levels {
            image_barrier.subresource_range.base_mip_level = i - 1;
            image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            image_barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            image_barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

            unsafe {
                self.device.get_device()
                    .cmd_pipeline_barrier(
                        self.cmd_buffer,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[image_barrier.clone()],
                    );
            }

            let blits = [vk::ImageBlit {
                src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i - 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                src_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: mip_width,
                        y: mip_height,
                        z: 1,
                    },
                ],
                dst_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                dst_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: max(mip_width / 2, 1),
                        y: max(mip_height / 2, 1),
                        z: 1,
                    },
                ],
            }];

            unsafe {
                self.device.get_device()
                    .cmd_blit_image(
                        self.cmd_buffer,
                        image.get_image(),
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        image.get_image(),
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &blits,
                        vk::Filter::LINEAR,
                    );
            }

            image_barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
            image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            unsafe {
                self.device.get_device()
                    .cmd_pipeline_barrier(
                        self.cmd_buffer,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[image_barrier.clone()],
                    );
            }

            mip_width = max(mip_width / 2, 1);
            mip_height = max(mip_height / 2, 1);
        }

        image_barrier.subresource_range.base_mip_level = mip_levels - 1;
        image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            self.device.get_device()
                .cmd_pipeline_barrier(
                    self.cmd_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier.clone()],
                );
        }
    }

    pub fn set_desc_layout(&mut self,
        set: u32,
        layout: Arc<VkDescriptorSetLayout>
    ) {
        self.desc_layouts.insert(set, layout);
    }

    fn get_desc_set(&mut self, set: u32) -> Arc<VkDescriptorSet> {
        let desc_layout = self.desc_layouts.get(&set)
                                                                    .expect("Failed to set desc buffer. (Missing desc layout_");

        let desc_set = match self.desc_sets.get(&set) {
            Some(desc_set) => desc_set,
            None => {
                let desc_set = VkDescriptorSet::new(
                    self.device.clone(),
                    self.desc_pool.clone(),
                    desc_layout.clone(),

                );
                self.desc_sets.insert(set, desc_set);
                self.desc_sets.get(&set).as_ref().unwrap()
            }
        };

        desc_set.clone()
    }

    pub fn set_desc_buffer(&mut self,
        set: u32,
        binding: u32,
        desc_type: vk::DescriptorType,
        uniform_buffer: RcCell<VkUniformBuffer>
    ) {
        let desc_set = self.get_desc_set(set);

        let descriptor_buffer_info = [vk::DescriptorBufferInfo {
            buffer: uniform_buffer.as_ref().get_buffer(),
            offset: 0,
            range: uniform_buffer.as_ref().size() as u64,
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

        self.tracked_buffers.push(uniform_buffer.as_ref().track_buffer());
    }

    pub fn set_desc_texture(&mut self,
        set: u32,
        binding: u32,
        sampler: &VkSampler,
        texture: &mut VkTexture,
        image_layout: vk::ImageLayout
    ) {
        let desc_set = self.get_desc_set(set);

        let descriptor_image_infos = [vk::DescriptorImageInfo {
            sampler: sampler.get_sampler(),
            image_view: texture.get_image_view(),
            image_layout: image_layout,
        }];

        let descriptor_write_sets = [
            vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: std::ptr::null(),
                dst_set: desc_set.get_desc_set(),
                dst_binding: binding,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                p_image_info: descriptor_image_infos.as_ptr(),
                p_buffer_info: std::ptr::null(),
                p_texel_buffer_view: std::ptr::null(),
            }
        ];

        unsafe {
            self.device.get_device()
                .update_descriptor_sets(&descriptor_write_sets, &[]);
        }
    }

    pub fn set_desc_img(&mut self,
        set: u32,
        binding: u32,
        texture: &mut VkImage,
        image_layout: vk::ImageLayout
    ) {
        let desc_set = self.get_desc_set(set);

        let descriptor_image_infos = [vk::DescriptorImageInfo {
            sampler: vk::Sampler::default(),
            image_view: texture.get_image_view(),
            image_layout: image_layout,
        }];

        let descriptor_write_sets = [
            vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: std::ptr::null(),
                dst_set: desc_set.get_desc_set(),
                dst_binding: binding,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
                p_image_info: descriptor_image_infos.as_ptr(),
                p_buffer_info: std::ptr::null(),
                p_texel_buffer_view: std::ptr::null(),
            }
        ];

        unsafe {
            self.device.get_device()
                .update_descriptor_sets(&descriptor_write_sets, &[]);
        }
    }

    pub fn set_desc_tlas(&mut self,
        set: u32,
        binding: u32,
        tlas: &VkTlas
    ) {
        let desc_set = self.get_desc_set(set);

        let accel_info = vk::WriteDescriptorSetAccelerationStructureKHR {
            acceleration_structure_count: 1,
            p_acceleration_structures: &tlas.get_accel(),
            ..Default::default()
        };

        let descriptor_write_sets = [
            vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: &accel_info as *const vk::WriteDescriptorSetAccelerationStructureKHR as *const std::ffi::c_void,
                dst_set: desc_set.get_desc_set(),
                dst_binding: binding,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
                p_image_info: std::ptr::null(),
                p_buffer_info: std::ptr::null(),
                p_texel_buffer_view: std::ptr::null(),
            }
        ];

        unsafe {
            self.device.get_device()
                .update_descriptor_sets(&descriptor_write_sets, &[]);
        }
    }

    pub fn bind_desc_sets(&mut self) {
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

        for desc_set in sorted_desc_sets {
            self.tracked_desc_sets.push(desc_set.1.clone());
        }
        self.desc_sets.clear();
    }

    pub fn push_constant<T: Sized>(&self, constant: &T, stage_flags: vk::ShaderStageFlags) {
        let pipeline = self.pipeline.as_ref()
                                        .expect("Failed to push constant. (No pipeline bound)").as_ref();

        unsafe {
            let bytes = core::slice::from_raw_parts(
                (constant as *const T) as *const u8,
                core::mem::size_of::<T>()
            );

            self.device.get_device()
                .cmd_push_constants(
                    self.cmd_buffer,
                    pipeline.get_layout(),
                    stage_flags,
                    0,
                    bytes
                );
        }
    }

    pub fn barrier(&self,
        src_access_mask: vk::AccessFlags,
        dst_access_mask: vk::AccessFlags,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags
    ) {
        let barrier = vk::MemoryBarrier::builder()
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .build();

        unsafe {
            self.device.get_device()
                .cmd_pipeline_barrier(
                    self.cmd_buffer,
                    src_stage_mask,
                    dst_stage_mask,
                    vk::DependencyFlags::empty(),
                    &[barrier],
                    &[],
                    &[]
                );
        }
    }

    pub fn create_blas(&self,
        build_infos: &mut Vec<VkAccelBuildInfo>,
        indices: &Vec<usize>,
        scratch_address: vk::DeviceAddress,
        query_pool: Option<Arc<VkQueryPool>>
    ) {
        if let Some(query_pool) = &query_pool {
            query_pool.reset();
        }

        for i in indices {
            let build_info = &mut build_infos[*i];

            let mut create_info = vk::AccelerationStructureCreateInfoKHR::builder()
                .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .size(build_info.size_info.acceleration_structure_size)
                .build();

            build_info.accel = Some(Arc::new(VkAccel::new(
                self.device.clone(),
                self.allocator.clone(),
                &mut create_info
            )));
            
            { let mut build_info_mut = build_info.build_info.as_mut();
                build_info_mut.dst_acceleration_structure = build_info.accel.as_ref().unwrap().get_accel();
                build_info_mut.scratch_data.device_address = scratch_address;
            }

            unsafe {
                self.device.accel_loader()
                    .cmd_build_acceleration_structures(
                        self.cmd_buffer,
                        &[*build_info.build_info.as_ref()],
                        &[build_info.get_range_info()]
                    );
            }

            self.barrier(
                vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
                vk::AccessFlags::ACCELERATION_STRUCTURE_READ_KHR,
                vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR
            );

            unsafe {
                if let Some(query_pool) = &query_pool {
                    self.device.accel_loader()
                        .cmd_write_acceleration_structures_properties(
                            self.cmd_buffer,
                            &[build_info.build_info.as_ref().dst_acceleration_structure],
                            vk::QueryType::ACCELERATION_STRUCTURE_COMPACTED_SIZE_KHR,
                            query_pool.get_query_pool(),
                            0
                        );
                }
            }
        }
    }

    pub fn compact_blas(&self,
        build_infos: &mut Vec<VkAccelBuildInfo>,
        indices: &Vec<usize>,
        query_pool: Arc<VkQueryPool>
    ) {
        let mut query_count = 0;
        let compact_sizes = query_pool.query_results::<vk::DeviceSize>(indices.len());
        
        for i in indices {
            let build_info = &mut build_infos[*i];

            build_info.cleanup = build_info.accel.clone();
            build_info.size_info.acceleration_structure_size = compact_sizes[query_count];
            query_count += 1;

            let mut create_info = vk::AccelerationStructureCreateInfoKHR::builder()
                .size(build_info.size_info.acceleration_structure_size)
                .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .build();
            build_info.accel = Some(Arc::new(VkAccel::new(
                self.device.clone(),
                self.allocator.clone(),
                &mut create_info
            )));

            let copy_info = vk::CopyAccelerationStructureInfoKHR::builder()
                .src(build_info.build_info.as_ref().dst_acceleration_structure)
                .dst(build_info.accel.as_ref().unwrap().get_accel())
                .mode(vk::CopyAccelerationStructureModeKHR::COMPACT)
                .build();
            unsafe {
                self.device.accel_loader()
                    .cmd_copy_acceleration_structure(
                        self.cmd_buffer,
                        &copy_info
                    );
            }
        }
    }

    pub fn create_tlas(&mut self,
        tlas: Option<ArcMutex<VkAccel>>,
        instance_count: u32,
        instance_buffer_address: vk::DeviceAddress,
        build_flags: vk::BuildAccelerationStructureFlagsKHR,
        update: bool,
        accel_props: &vk::PhysicalDeviceAccelerationStructurePropertiesKHR
    ) -> Option<ArcMutex<VkAccel>> {
        assert!(!(tlas.is_none() && update), "Failed to create tlas.");

        let instance_data = vk::AccelerationStructureGeometryInstancesDataKHR::builder()
            .data(vk::DeviceOrHostAddressConstKHR {
                device_address: instance_buffer_address
            })
            .build();

        let top_as_geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::INSTANCES)
            .geometry(vk::AccelerationStructureGeometryDataKHR {
                instances: instance_data
            })
            .build();
        let geometries = vec![top_as_geometry];

        let mode = if update {
            vk::BuildAccelerationStructureModeKHR::UPDATE
        } else {
            vk::BuildAccelerationStructureModeKHR::BUILD
        };
        
        let mut build_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .flags(build_flags)
            .geometries(&geometries)
            .mode(mode)
            .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
            .build();

        let size_info = unsafe {
            self.device.accel_loader()
                .get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::DEVICE,
                    &build_info,
                    &[instance_count]
                )
        };

        let tlas = if !update {
            let mut create_info = vk::AccelerationStructureCreateInfoKHR::builder()
                .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
                .size(size_info.acceleration_structure_size)
                .build();

            ArcMutex::new(VkAccel::new(
                self.device.clone(),
                self.allocator.clone(),
                &mut create_info
            ))
        } else {
            tlas.as_ref().unwrap().clone()
        };

        let scratch_buffer = Arc::new(VkBuffer::new(
            "Tlas SCRATCH BUFFER".to_owned(),
            self.device.clone(),
            self.allocator.clone(),
            size_info.build_scratch_size,
            vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            Some(accel_props.min_acceleration_structure_scratch_offset_alignment as u64)
        ));
        self.tracked_buffers.push(scratch_buffer.clone());
        let scratch_address = scratch_buffer.get_device_address();

        if update {
            build_info.src_acceleration_structure = tlas.as_ref().get_accel();
        }
        build_info.dst_acceleration_structure = tlas.as_ref().get_accel();
        build_info.scratch_data.device_address = scratch_address;

        let range_info = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .primitive_count(instance_count)
            .build();

        unsafe {
            self.device.accel_loader()
                .cmd_build_acceleration_structures(
                    self.cmd_buffer,
                    &[build_info],
                    &[&[range_info]]
                );
        }

        if !update {
            Some(tlas.clone())
        } else {
            None
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