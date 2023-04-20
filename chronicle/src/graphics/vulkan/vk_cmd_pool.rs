use std::ptr;

use ash::vk;

use crate::graphics::*;

pub struct VkCmdPool {
    device: Arc<VkLogicalDevice>,
    cmd_pool: vk::CommandPool
}

impl VkCmdPool {
    pub fn new(device: Arc<VkLogicalDevice>) -> Arc<Self> {
        let command_pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: device.get_queue_family_indices().graphics_family.unwrap(),
        };

        let cmd_pool = unsafe {
            device.get_device()
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create Command Pool.")
        };

        Arc::new(VkCmdPool {
            device: device,
            cmd_pool: cmd_pool
        })
    }

    pub fn get_cmd_pool(&self) -> vk::CommandPool {
        self.cmd_pool
    }
}

impl Drop for VkCmdPool {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .destroy_command_pool(self.cmd_pool, None);
        }
    }
}