use std::{rc::Rc, ffi::c_void};

use ash::vk;

use crate::graphics::*;

pub struct VkBuffer {
    device: Rc<VkLogicalDevice>,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize
}

impl VkBuffer {
    pub fn new(
        device: Rc<VkLogicalDevice>,
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

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            allocation_size: mem_requirements.size,
            memory_type_index: memory_type,
        };

        let buffer_memory = unsafe {
            device.get_device()
                .allocate_memory(&allocate_info, None)
                .expect("Failed to allocate vertex buffer memory.")
        };

        unsafe {
            device.get_device()
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Failed to bind Buffer.");
        }

        VkBuffer {
            device: device,
            buffer: buffer,
            memory: buffer_memory,
            size: size
        }
    }

    fn find_memory_type(
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
        unsafe {
            self.device.get_device()
                .map_memory(
                    self.memory,
                    0,
                    self.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory.")
        }
    }

    pub fn unmap(&self) {
        unsafe {
            self.device.get_device()
                .unmap_memory(self.memory);
        }
    }

    pub fn get_buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn get_memory(&self) -> vk::DeviceMemory {
        self.memory
    }
}

impl Drop for VkBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .free_memory(self.memory, None);

            self.device.get_device()
                .destroy_buffer(self.buffer, None);
        }
    }
}