use std::rc::Rc;
use std::ptr;

use ash::version::DeviceV1_0;
use ash::vk;

use crate::graphics::*;

pub struct VkFence {
    device: Rc<VkLogicalDevice>,
    fence: vk::Fence
}

impl VkFence {
    pub fn new(device: Rc<VkLogicalDevice>) -> Self {
        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED,
        };

        let fence = unsafe { device.get_device()
            .create_fence(&fence_create_info, None)
            .expect("Failed to create Semaphore Object.")
        };

        VkFence {
            device: device,
            fence: fence
        }
    }

    pub fn get_fence(&self) -> &vk::Fence {
        &self.fence
    }

    pub fn wait(&self) {
        unsafe {
            self.device.get_device()
                .wait_for_fences(&[self.fence], true, std::u64::MAX)
                .expect("Failed to wait for Fence.");
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.device.get_device()
                .reset_fences(&[self.fence])
                .expect("Failed to reset Fence.");
        }
    }
}

impl Drop for VkFence {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .destroy_fence(self.fence, None);
        }
    }
}