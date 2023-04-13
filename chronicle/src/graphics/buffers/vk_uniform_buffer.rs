use std::rc::Rc;

use ash::vk;

use crate::graphics::*;

pub struct VkUniformBuffer<T: Default> {
    uniform_buffer: VkBuffer,
    data: *mut T
}

impl<T: Default> VkUniformBuffer<T> {
    pub fn new(device: Rc<VkLogicalDevice>, physical_device: &VkPhysicalDevice) -> Self {
        let uniform_buffer = VkBuffer::new(
            device,
            std::mem::size_of::<T>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            physical_device.get_mem_properties()
        );

        let data = uniform_buffer.map() as *mut T;

        VkUniformBuffer {
            uniform_buffer: uniform_buffer,
            data: data
        }
    }

    pub fn get_buffer(&self) -> vk::Buffer {
        self.uniform_buffer.get_buffer()
    }

    pub fn data(&mut self) -> &mut T {
        unsafe { self.data.as_mut().unwrap() }
    }
}

impl<T: Default> Drop for VkUniformBuffer<T> {
    fn drop(&mut self) {
        self.uniform_buffer.unmap();
    }
}