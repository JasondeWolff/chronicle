use ash::vk;

use crate::graphics::*;

pub struct VkDescriptorSet {
    device: Arc<VkLogicalDevice>,
    desc_pool: Arc<VkDescriptorPool>,
    descriptor_set: vk::DescriptorSet
}

impl VkDescriptorSet {
    pub fn new(
        device: Arc<VkLogicalDevice>,
        desc_pool: Arc<VkDescriptorPool>,
        desc_layout: Arc<VkDescriptorSetLayout>
    ) -> Arc<Self> {
        let desc_layouts = [desc_layout.get_desc_layout()];

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            descriptor_pool: desc_pool.get_desc_pool(),
            descriptor_set_count: 1,
            p_set_layouts: desc_layouts.as_ptr()
        };

        let descriptor_sets = unsafe {
            device.get_device()
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets.")
        };

        Arc::new(VkDescriptorSet {
            device: device,
            desc_pool: desc_pool,
            descriptor_set: descriptor_sets[0]
        })
    }

    pub fn get_desc_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }
}

impl Drop for VkDescriptorSet {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .free_descriptor_sets(
                    self.desc_pool.get_desc_pool(),
                    &[self.descriptor_set]
                )
                .expect("Failed to free descriptor set.");
        }
    }
}