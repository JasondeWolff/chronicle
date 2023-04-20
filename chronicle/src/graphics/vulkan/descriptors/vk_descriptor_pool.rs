use std::rc::Rc;

use ash::vk;

use crate::graphics::*;

// TODO: Implement desc pooling and resetting here? Instead of destroying and recreating them every frame
// also remove VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT then
// make descriptor sets only obtainable by using a desc pool
// descriptor set layout should be hashable to reuse old descriptors

pub struct VkDescriptorPool {
    device: Arc<VkLogicalDevice>,
    desc_pool: vk::DescriptorPool
}

impl VkDescriptorPool {
    pub fn new(device: Arc<VkLogicalDevice>) -> Arc<Self> {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 64,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 128,//32,
            }
        ];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET,
            max_sets: 128,//16,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
        };

        let desc_pool = unsafe {
            device.get_device()
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to create Descriptor Pool.")
        };

        Arc::new(VkDescriptorPool {
            device: device,
            desc_pool: desc_pool
        })
    }

    pub fn get_desc_pool(&self) -> vk::DescriptorPool {
        self.desc_pool
    }
}

impl Drop for VkDescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                    .destroy_descriptor_pool(self.desc_pool, None);
        }
    }
}