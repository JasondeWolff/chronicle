use std::rc::Rc;

use ash::vk;

use crate::graphics::*;

pub struct VkSampler {
    device: Rc<VkLogicalDevice>,
    sampler: vk::Sampler
}

impl VkSampler {
    pub fn new(device: Rc<VkLogicalDevice>) -> Self {
        let sampler_create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            mip_lod_bias: 0.0,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: 16.0,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            min_lod: 0.0,
            max_lod: 0.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
        };

        let sampler = unsafe {
            device.get_device()
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler.")
        };

        VkSampler {
            device: device,
            sampler: sampler
        }
    }

    pub fn get_sampler(&self) -> vk::Sampler {
        self.sampler
    }
}

impl Drop for VkSampler {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .destroy_sampler(self.sampler, None);
        }
    }
}