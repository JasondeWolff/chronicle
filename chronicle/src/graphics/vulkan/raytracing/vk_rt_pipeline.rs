use std::ptr;

use crate::graphics::*;

fn align_up(size: u32, alignment: u32) -> u32 {
    (size + (alignment - 1)) & !(alignment - 1)
}

pub struct VkRTPipeline {
    device: Arc<VkLogicalDevice>,
    shader_stage_indices: HashMap<vk::ShaderStageFlags, usize>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    sbt: Arc<VkBuffer>
}

impl VkRTPipeline {
    pub fn new(
        device: Arc<VkLogicalDevice>,
        allocator: ArcMutex<Allocator>,
        rt_properties: &vk::PhysicalDeviceRayTracingPipelinePropertiesKHR,
        desc_layouts: &Vec<&VkDescriptorSetLayout>,
        push_constants: &Vec<vk::PushConstantRange>,
        shaders: &Vec<String>,
        max_ray_recursion_depth: u32
    ) -> Arc<Self> {
        let main_function_name = std::ffi::CString::new("main").unwrap();

        let mut shader_modules = Vec::new();
        let mut shader_stages = Vec::new();
        let mut shader_stage_indices = HashMap::new();
        for (i, shader) in shaders.iter().enumerate() {
            let shader_module = VkShaderModule::new(device.clone(), shader.clone());
            shader_stages.push(vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: *shader_module.get_module(),
                p_name: main_function_name.as_ptr(),
                p_specialization_info: std::ptr::null(),
                stage: *shader_module.get_stage_flags()
            });
            shader_stage_indices.insert(*shader_module.get_stage_flags(), i);
            shader_modules.push(shader_module);
        }

        let mut shader_groups = Vec::new();
        for (stage_flags, i) in &shader_stage_indices {
            let mut shader_group_info = vk::RayTracingShaderGroupCreateInfoKHR::builder()
                .any_hit_shader(vk::SHADER_UNUSED_KHR)
                .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                .general_shader(vk::SHADER_UNUSED_KHR)
                .intersection_shader(vk::SHADER_UNUSED_KHR)
                .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
                .general_shader(*i as u32)
                .build();

            match *stage_flags {
                vk::ShaderStageFlags::CLOSEST_HIT_KHR => {
                    shader_group_info.ty = vk::RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP;
                    shader_group_info.general_shader = vk::SHADER_UNUSED_KHR;
                    shader_group_info.closest_hit_shader = *i as u32;
                }
                _ => {}
            }

            shader_groups.push(shader_group_info);
        }

        let mut set_layouts = Vec::new();
        for desc_layout in desc_layouts {
            set_layouts.push(desc_layout.get_desc_layout());
        }

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: desc_layouts.len() as u32,
            p_set_layouts: set_layouts.as_ptr(),
            push_constant_range_count: push_constants.len() as u32,
            p_push_constant_ranges: push_constants.as_ptr(),
        };

        let pipeline_layout = unsafe {
            device.get_device()
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout.")
        };

        let rt_pipeline_info = vk::RayTracingPipelineCreateInfoKHR::builder()
            .stages(&shader_stages)
            .groups(&shader_groups)
            .max_pipeline_ray_recursion_depth(max_ray_recursion_depth)
            .layout(pipeline_layout)
            .build();
        
        let pipeline = unsafe {
            device.raytracing_loader()
                .create_ray_tracing_pipelines(
                    vk::DeferredOperationKHR::default(),
                    vk::PipelineCache::default(),
                    &vec![rt_pipeline_info],
                    None
                ).expect("Failed to create rt pipeline.")
        };

        let sbt = Arc::new(Self::create_sbt(
            device.clone(),
            allocator,
            pipeline[0],
            rt_properties,
            1, 1
        ));

        Arc::new(
            VkRTPipeline {
                device: device,
                shader_stage_indices: shader_stage_indices,
                pipeline_layout: pipeline_layout,
                pipeline: pipeline[0],
                sbt: sbt
            }
        )
    }

    fn create_sbt(
        device: Arc<VkLogicalDevice>,
        allocator: ArcMutex<Allocator>,
        pipeline: vk::Pipeline,
        rt_properties: &vk::PhysicalDeviceRayTracingPipelinePropertiesKHR,
        miss_count: u32, hit_count: u32
    ) -> VkBuffer {
        let handle_count = 1 + miss_count + hit_count;
        let handle_size = rt_properties.shader_group_handle_size;
        let handle_size_aligned = align_up(handle_size, rt_properties.shader_group_base_alignment);

        let mut rgen_region = vk::StridedDeviceAddressRegionKHR::default();
        let mut miss_region = vk::StridedDeviceAddressRegionKHR::default();
        let mut hit_region = vk::StridedDeviceAddressRegionKHR::default();
        let call_region = vk::StridedDeviceAddressRegionKHR::default();

        rgen_region.stride = align_up(handle_size_aligned, rt_properties.shader_group_base_alignment) as u64;
        rgen_region.size = rgen_region.stride;
        miss_region.stride = handle_size_aligned as u64;
        miss_region.size = align_up(miss_count * handle_size_aligned, rt_properties.shader_group_base_alignment) as u64;
        hit_region.stride = handle_size_aligned as u64;
        hit_region.size = align_up(hit_count * handle_size_aligned, rt_properties.shader_group_base_alignment) as u64;
    
        let data_size = (handle_count * handle_size) as usize;
        let mut handles = unsafe {
            device.raytracing_loader()
                .get_ray_tracing_shader_group_handles(
                    pipeline,
                    0, handle_count,
                    data_size
                )
                .expect("Failed to get rt shader group handles.")
        };

        let sbt_size = rgen_region.size + miss_region.size + hit_region.size + call_region.size;
        let sbt = VkBuffer::new(
            "Shader Binding Table".to_owned(),
            device,
            allocator,
            sbt_size,
            vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            None
        );
        let sbt_address = sbt.get_device_address();

        rgen_region.device_address = sbt_address;
        miss_region.device_address = sbt_address + rgen_region.size;
        hit_region.device_address = sbt_address + rgen_region.size + miss_region.size;

        unsafe {
            let mut get_handle = |i: isize| -> *mut u8  {
                let begin = &mut handles[0] as *mut u8;
                begin.offset(i)
            };
            let mut handle_idx = 0;

            let data_ptr = sbt.map() as *mut u8;

            data_ptr.offset(0).copy_from_nonoverlapping(get_handle(handle_idx), handle_size as usize);
            handle_idx += 1;
            for i in 0..miss_count {
                let offset = rgen_region.size + i as u64 * miss_region.stride;
                data_ptr.offset(offset as isize).copy_from_nonoverlapping(get_handle(handle_idx), handle_size as usize);
                handle_idx += 1;
            }
            for i in 0..hit_count {
                let offset = rgen_region.size + miss_region.size + i as u64 * hit_region.stride;
                data_ptr.offset(offset as isize).copy_from_nonoverlapping(get_handle(handle_idx), handle_size as usize);
                handle_idx += 1;
            }

            sbt.unmap();
        }

        sbt
    }

    pub fn get_stage_index(&self, stage_flags: &vk::ShaderStageFlags) -> u32 {
        *self.shader_stage_indices.get(stage_flags)
            .expect("Failed to get stage index. (Shader stage not available)") as u32
    }

    pub fn get_pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn get_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
}

impl Drop for VkRTPipeline {
    fn drop(&mut self) {
        unsafe {
            
        }
    }
}