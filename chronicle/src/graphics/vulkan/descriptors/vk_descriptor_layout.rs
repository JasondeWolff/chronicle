use std::rc::Rc;

use ash::vk;

use crate::graphics::*;

#[derive(Clone)]
pub struct VkDescriptorLayout {
    device: Rc<VkLogicalDevice>,
    desc_layout: vk::DescriptorSetLayout
}

impl VkDescriptorLayout {
    pub fn new(device: Rc<VkLogicalDevice>) -> Self {
        let ubo_layout_bindings = [
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                p_immutable_samplers: std::ptr::null(),
            },
            // vk::DescriptorSetLayoutBinding {
            //     binding: 1,
            //     descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            //     descriptor_count: 1,
            //     stage_flags: vk::ShaderStageFlags::FRAGMENT,
            //     p_immutable_samplers: std::ptr::null(),
            // },
        ];

        let ubo_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: ubo_layout_bindings.len() as u32,
            p_bindings: ubo_layout_bindings.as_ptr(),
        };

        let desc_layout = unsafe {
            device.get_device()
                .create_descriptor_set_layout(&ubo_layout_create_info, None)
                .expect("Failed to create Descriptor Set Layout.")
        };

        VkDescriptorLayout {
            device: device,
            desc_layout: desc_layout
        }
    }

    pub fn get_desc_layout(&self) -> vk::DescriptorSetLayout {
        self.desc_layout
    }
}

impl Drop for VkDescriptorLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .destroy_descriptor_set_layout(self.desc_layout, None);
        }
    }
}