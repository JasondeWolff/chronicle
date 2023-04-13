use std::rc::Rc;
use std::collections::HashMap;

use ash::vk;

use crate::graphics::*;

pub struct VkDescriptorPool {
    device: Rc<VkLogicalDevice>,
    descriptor_pools: HashMap<vk::DescriptorType, vk::DescriptorPool>
}

impl VkDescriptorPool {
    pub fn new(device: Rc<VkLogicalDevice>) -> Self {
        VkDescriptorPool {
            device: device,
            descriptor_pools: HashMap::new()
        }
    }

    pub fn get_desc_pool(&mut self, desc_type: vk::DescriptorType) -> vk::DescriptorPool {
        match self.descriptor_pools.get(&desc_type) {
            Some(desc_pool) => *desc_pool,
            None => {
                let desc_pool = self.create_desc_pool(desc_type);
                self.descriptor_pools.insert(desc_type, desc_pool);
                desc_pool
            }
        }
    }

    fn create_desc_pool(&self, desc_type: vk::DescriptorType) -> vk::DescriptorPool {
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 3 as u32,
        }];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: pool_sizes[0].descriptor_count,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
        };

        unsafe {
            self.device.get_device()
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