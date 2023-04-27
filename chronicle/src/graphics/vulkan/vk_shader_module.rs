use crate::graphics::*;
use crate::app;

pub struct VkShaderModule {
    device: Arc<VkLogicalDevice>,
    shader_module: vk::ShaderModule,
    shader_stage_flags: vk::ShaderStageFlags
}

impl VkShaderModule {
    pub fn new(device: Arc<VkLogicalDevice>, name: String) -> Self {
        let shader_code = app().resources()
            .get_binary_blob(format!("assets/builtin/shaders/bin/{name}.spv"));

        let create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: shader_code.as_ref().len(),
            p_code: shader_code.as_ref().as_ptr() as *const u32,
        };

        let shader_module = unsafe {
            device.get_device().create_shader_module(&create_info, None)
                .expect("Failed to create Shader Module!")
        };

        let shader_stage_flags = match std::path::Path::new(&name).extension()
                                                        .expect("Failed to get shader type from file extension").to_str().unwrap() {
            "vert" | "vertex" | "vs" => vk::ShaderStageFlags::VERTEX,
            "frag" | "fragment" | "fs" => vk::ShaderStageFlags::FRAGMENT,
            "tesc" | "tessellation_control" | "tcs" => vk::ShaderStageFlags::TESSELLATION_CONTROL,
            "tese" | "tessellation_evaluation" | "tes" => vk::ShaderStageFlags::TESSELLATION_EVALUATION,
            "geom" | "geometry" | "gs" => vk::ShaderStageFlags::GEOMETRY,
            "comp" | "compute" | "cs" => vk::ShaderStageFlags::COMPUTE,
            "rgen" => vk::ShaderStageFlags::RAYGEN_KHR,
            "rmiss" => vk::ShaderStageFlags::MISS_KHR,
            "rchit" => vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            extension => panic!("Failed to get shader type from file extension, unable to recognize \"{extension}\".")
        };

        VkShaderModule {
            device: device,
            shader_module: shader_module,
            shader_stage_flags: shader_stage_flags
        }
    }

    pub fn get_module(&self) -> &vk::ShaderModule {
        &self.shader_module
    }

    pub fn get_stage_flags(&self) -> &vk::ShaderStageFlags {
        &self.shader_stage_flags
    }
}

impl Drop for VkShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .destroy_shader_module(self.shader_module, None);
        }
    }
}