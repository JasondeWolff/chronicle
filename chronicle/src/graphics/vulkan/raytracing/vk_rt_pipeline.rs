use std::ptr;

use crate::graphics::*;

pub struct VkRTPipeline {
    device: Arc<VkLogicalDevice>,
    shader_stage_indices: HashMap<vk::ShaderStageFlags, usize>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline
}

impl VkRTPipeline {
    pub fn new(
        device: Arc<VkLogicalDevice>,
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

        Arc::new(
            VkRTPipeline {
                device: device,
                shader_stage_indices: shader_stage_indices,
                pipeline_layout: pipeline_layout,
                pipeline: pipeline[0]
            }
        )
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