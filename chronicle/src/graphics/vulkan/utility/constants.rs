use crate::graphics::utility::debug::ValidationInfo;
use ash::vk::make_api_version;

pub const APPLICATION_VERSION: u32 = make_api_version(0, 1, 0, 0);

pub const ENGINE_TITLE: &'static str = "Chronicle";
pub const ENGINE_VERSION: u32 = make_api_version(0, 1, 0, 0);
pub const API_VERSION: u32 = make_api_version(0, 1, 3, 92);

pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

// Depenedents chain down E.G.
// "VK_KHR_A"
// "VK_KHR_B" (required for VK_KHR_A)
// "VK_KHR_C" (required for VK_KHR_B)
pub const DEVICE_EXTENSIONS: [&'static str; 9] = [
    "VK_KHR_swapchain",

    "VK_KHR_device_group",
    "VK_KHR_buffer_device_address",

    "VK_KHR_acceleration_structure",
    "VK_EXT_descriptor_indexing",

    "VK_KHR_ray_tracing_pipeline",

    "VK_KHR_deferred_host_operations",
    "VK_KHR_spirv_1_4",
    "VK_KHR_shader_float_controls"
];

pub const ENABLE_EXTENSION_NAMES: [*const std::ffi::c_char; 9] = [
    ash::extensions::khr::Swapchain::name().as_ptr(),
    ash::extensions::khr::DeviceGroup::name().as_ptr(),
    ash::extensions::khr::BufferDeviceAddress::name().as_ptr(),
    ash::extensions::khr::AccelerationStructure::name().as_ptr(),
    ash::vk::ExtDescriptorIndexingFn::name().as_ptr(),
    ash::extensions::khr::RayTracingPipeline::name().as_ptr(),
    ash::extensions::khr::DeferredHostOperations::name().as_ptr(),
    ash::vk::KhrSpirv14Fn::name().as_ptr(),
    ash::vk::KhrShaderFloatControlsFn::name().as_ptr()
];

pub const MAX_FRAMES_IN_FLIGHT: usize = 3;