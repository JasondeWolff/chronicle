use std::rc::Rc;
use std::ptr;

use ash::vk;

use crate::graphics::*;

pub struct VkFence {
    device: Rc<VkLogicalDevice>,
    fence: vk::Fence
}

impl VkFence {
    pub fn new(device: Rc<VkLogicalDevice>, signaled: bool) -> Rc<Self> {
        let create_flags = if signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            vk::FenceCreateFlags::default()
        };

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: create_flags
        };

        let fence = unsafe { device.get_device()
            .create_fence(&fence_create_info, None)
            .expect("Failed to create Semaphore Object.")
        };

        Rc::new(VkFence {
            device: device,
            fence: fence
        })
    }

    pub fn get_fence(&self) -> vk::Fence {
        self.fence
    }

    pub fn is_completed(&self) -> bool {
        unsafe {
            self.device.get_device()
                .get_fence_status(self.fence)
                .expect("Failed to get Fence status.")
        }
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