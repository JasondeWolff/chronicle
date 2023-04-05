use crate::utility::debug::ValidationInfo;
use ash::vk_make_version;

pub const APPLICATION_VERSION: u32 = vk_make_version!(1, 0, 0);

pub const ENGINE_TITLE: &'static str = "Chronicle";
pub const ENGINE_VERSION: u32 = vk_make_version!(1, 0, 0);
pub const API_VERSION: u32 = vk_make_version!(1, 0, 92);

pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;