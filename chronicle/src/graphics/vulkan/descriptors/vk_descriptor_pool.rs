use std::rc::Rc;
use std::collections::HashMap;

use ash::vk;

use crate::graphics::*;

const DESCRIPTOR_TYPES: [vk::DescriptorType; 11] = [
    vk::DescriptorType::SAMPLER,
    vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
    vk::DescriptorType::SAMPLED_IMAGE,
    vk::DescriptorType::STORAGE_IMAGE,
    vk::DescriptorType::UNIFORM_TEXEL_BUFFER,
    vk::DescriptorType::STORAGE_TEXEL_BUFFER,
    vk::DescriptorType::UNIFORM_BUFFER,
    vk::DescriptorType::STORAGE_BUFFER,
    vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
    vk::DescriptorType::STORAGE_BUFFER_DYNAMIC,
    vk::DescriptorType::INPUT_ATTACHMENT
];

// TODO: Implement desc pooling and resetting here? Instead of destroying and recreating them every frame
// also remove VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT then
// make descriptor sets only obtainable by using a desc pool
// descriptor set layout should be hashable to reuse old descriptors

pub struct VkDescriptorPool {
    device: Rc<VkLogicalDevice>,
    descriptor_pools: HashMap<vk::DescriptorType, vk::DescriptorPool>
}

impl VkDescriptorPool {
    pub fn new(device: Rc<VkLogicalDevice>) -> Rc<Self> {
        let mut descriptor_pools = HashMap::new();
        for desc_type in DESCRIPTOR_TYPES {
            let desc_pool = Self::create_desc_pool(device.clone(), desc_type);
            descriptor_pools.insert(desc_type, desc_pool);
        }

        Rc::new(VkDescriptorPool {
            device: device,
            descriptor_pools: descriptor_pools
        })
    }

    pub fn get_desc_pool(&self, desc_type: vk::DescriptorType) -> vk::DescriptorPool {
        match self.descriptor_pools.get(&desc_type) {
            Some(desc_pool) => *desc_pool,
            None => panic!("Failed to get desc pool.")
        }
    }

    fn create_desc_pool(device: Rc<VkLogicalDevice>, desc_type: vk::DescriptorType) -> vk::DescriptorPool {
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: desc_type,
            descriptor_count: 64 as u32,
        }];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET,
            max_sets: pool_sizes[0].descriptor_count,
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
            for desc_pool in &self.descriptor_pools {
                self.device.get_device()
                    .destroy_descriptor_pool(*desc_pool.1, None);
            }
        }
    }
}