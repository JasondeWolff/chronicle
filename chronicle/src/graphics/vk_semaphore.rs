use std::rc::Rc;
use std::ptr;

use ash::version::DeviceV1_0;
use ash::vk;

use crate::graphics::*;

pub struct VkSemaphore {
    device: Rc<VkLogicalDevice>,
    semaphore: vk::Semaphore
}

impl VkSemaphore {
    pub fn new(device: Rc<VkLogicalDevice>) -> Rc<Self> {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };

        let semaphore = unsafe {
            device.get_device()
                .create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create Semaphore Object.")
        };

        Rc::new(VkSemaphore {
            device: device,
            semaphore: semaphore
        })
    }

    pub fn get_semaphore(&self) -> &vk::Semaphore {
        &self.semaphore
    }
}

impl Drop for VkSemaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .destroy_semaphore(self.semaphore, None);
        }
    }
}