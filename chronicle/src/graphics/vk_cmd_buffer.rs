use std::rc::Rc;
use std::ptr;

use ash::version::DeviceV1_0;
use ash::vk;

use crate::graphics::*;
use utility::constants::MAX_FRAMES_IN_FLIGHT;

pub struct VkCmdBuffer {
    device: Rc<VkLogicalDevice>,
    cmd_buffers: Vec<vk::CommandBuffer>
}

impl VkCmdBuffer {
    pub fn new(device: Rc<VkLogicalDevice>, cmd_pool: &VkCmdPool, ) -> Self {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: MAX_FRAMES_IN_FLIGHT as u32,
            command_pool: *cmd_pool.get_cmd_pool(),
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffers = unsafe {
            device.get_device()
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers.")
        };

        VkCmdBuffer {
            device: device,
            cmd_buffers: command_buffers
        }
    }

    pub fn get_cmd_buffer(&self, idx: usize)-> &vk::CommandBuffer {
        &self.cmd_buffers[idx]
    }
}