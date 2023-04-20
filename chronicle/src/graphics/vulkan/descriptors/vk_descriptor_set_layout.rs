use std::rc::Rc;

use ash::vk;

use crate::graphics::*;

#[derive(Clone)]
pub struct VkDescriptorSetLayout {
    device: Arc<VkLogicalDevice>,
    desc_layout: vk::DescriptorSetLayout
}

impl VkDescriptorSetLayout {
    pub fn new(
        device: Arc<VkLogicalDevice>,
        desc_layout_bindings: &Vec<vk::DescriptorSetLayoutBinding>
    ) -> Arc<Self> {
        let ubo_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: desc_layout_bindings.len() as u32,
            p_bindings: desc_layout_bindings.as_ptr(),
        };

        let desc_layout = unsafe {
            device.get_device()
                .create_descriptor_set_layout(&ubo_layout_create_info, None)
                .expect("Failed to create Descriptor Set Layout.")
        };

        Arc::new(VkDescriptorSetLayout {
            device: device,
            desc_layout: desc_layout
        })
    }

    pub fn get_desc_layout(&self) -> vk::DescriptorSetLayout {
        self.desc_layout
    }
}

impl Drop for VkDescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .destroy_descriptor_set_layout(self.desc_layout, None);
        }
    }
}