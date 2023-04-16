use std::rc::Rc;

use ash::vk;

use crate::graphics::*;

// TODO: Implement desc pooling and resetting here? Instead of destroying and recreating them every frame
// also remove VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT then
// make descriptor sets only obtainable by using a desc pool
// descriptor set layout should be hashable to reuse old descriptors

pub struct VkDescriptorPool {
    device: Rc<VkLogicalDevice>,
    desc_pool: vk::DescriptorPool
}

impl VkDescriptorPool {
    pub fn new(device: Rc<VkLogicalDevice>) -> Rc<Self> {
        let desc_pool = Self::create_desc_pool(device.clone());

        Rc::new(VkDescriptorPool {
            device: device,
            desc_pool: desc_pool
        })
    }

    pub fn get_desc_pool(&self) -> vk::DescriptorPool {
        self.desc_pool
    }

    fn create_desc_pool(device: Rc<VkLogicalDevice>) -> vk::DescriptorPool {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 64 as u32,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 32 as u32,
            }
        ];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET,
            max_sets: 16,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
        };

        unsafe {
            device.get_device()
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to create Descriptor Pool.")
        }
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