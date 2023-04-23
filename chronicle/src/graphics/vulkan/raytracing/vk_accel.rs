use ash::vk;

use crate::graphics::*;

pub struct VkAccel {
    device: Arc<VkLogicalDevice>,
    accel: vk::AccelerationStructureKHR,
    buffer: Arc<VkBuffer>
}

impl VkAccel {
    pub fn new(
        device: Arc<VkLogicalDevice>,
        allocator: ArcMutex<Allocator>,
        create_info: &mut vk::AccelerationStructureCreateInfoKHR
    ) -> Self {
        let buffer = Arc::new(VkBuffer::new(
            "Accel buffer",
            device.clone(),
            allocator,
            create_info.size,
            vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            None
        ));

        create_info.buffer = buffer.as_ref().get_buffer();

        let accel = unsafe {
            device.accel_loader()
                .create_acceleration_structure(create_info, None)
                    .expect("Failed to create acceleration structure.")
        };

        VkAccel {
            device,
            accel: accel,
            buffer: buffer
        }
    }

    pub fn get_accel(&self) -> vk::AccelerationStructureKHR {
        self.accel
    }

    pub fn get_accel_ref(&self) -> vk::AccelerationStructureReferenceKHR {
        let info = vk::AccelerationStructureDeviceAddressInfoKHR::builder()
            .acceleration_structure(self.accel)
            .build();

        let device_address = unsafe {
            self.device.accel_loader()
                .get_acceleration_structure_device_address(&info)
        };

        vk::AccelerationStructureReferenceKHR {
            device_handle: device_address
        }
    }
}