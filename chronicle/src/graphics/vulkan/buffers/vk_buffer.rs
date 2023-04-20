use std::{rc::Rc, ffi::c_void};

use ash::vk;

use crate::graphics::*;

pub struct VkBuffer {
    device: Arc<VkLogicalDevice>,
    allocator: ArcMutex<Allocator>,
    buffer: vk::Buffer,
    allocation: Option<Allocation>,
    size: vk::DeviceSize
}

impl VkBuffer {
    pub fn new(
        device: Arc<VkLogicalDevice>,
        allocator: ArcMutex<Allocator>,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        required_memory_properties: vk::MemoryPropertyFlags,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> Self {
        let buffer_create_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
        };

        let buffer = unsafe {
            device.get_device()
                .create_buffer(&buffer_create_info, None)
                .expect("Failed to create Vertex Buffer")
        };

        let mem_requirements = unsafe {
            device.get_device().get_buffer_memory_requirements(buffer)
        };
        let memory_type = Self::find_memory_type(
            mem_requirements.memory_type_bits,
            required_memory_properties,
            *device_memory_properties,
        );

        let location = if required_memory_properties.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL) {
            MemoryLocation::GpuOnly
        } else {
            MemoryLocation::CpuToGpu
        };

        let allocation = allocator.as_mut()
            .allocate(&AllocationCreateDesc {
                name: "VkBuffer",
                requirements: mem_requirements,
                location: location,
                linear: true,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged
            }).unwrap();

        unsafe {
            device.get_device()
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .expect("Failed to bind Buffer.");
        }

        VkBuffer {
            device: device,
            allocator: allocator,
            buffer: buffer,
            allocation: Some(allocation),
            size: size
        }
    }

    pub fn find_memory_type(
        type_filter: u32,
        required_properties: vk::MemoryPropertyFlags,
        mem_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> u32 {
        for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
            if (type_filter & (1 << i)) > 0
                && memory_type.property_flags.contains(required_properties)
            {
                return i as u32;
            }
        }

        panic!("Failed to find suitable memory type.")
    }

    pub fn map(&self) -> *mut c_void {
        unsafe { self.allocation.as_ref().unwrap().mapped_ptr().unwrap().as_mut() }
    }

    pub fn unmap(&self) {
    }

    pub fn get_buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn get_size(&self) -> vk::DeviceSize {
        self.size
    }
}

impl Drop for VkBuffer {
    fn drop(&mut self) {
        unsafe {
            if let Some(allocation) = self.allocation.take() {
                self.allocator.as_mut()
                    .free(allocation).unwrap();
            }

            self.device.get_device()
                .destroy_buffer(self.buffer, None);
        }
    }
}