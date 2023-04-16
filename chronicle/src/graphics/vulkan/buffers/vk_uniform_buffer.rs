use std::rc::Rc;

use ash::vk;

use crate::graphics::*;

pub struct VkUniformBuffer {
    uniform_buffer: Rc<VkBuffer>,
    data: *mut dyn ToAny,
    size: usize
}

impl VkUniformBuffer {
    pub fn new<T: ToAny>(device: Rc<VkLogicalDevice>, physical_device: &VkPhysicalDevice) -> Self {
        let size = std::mem::size_of::<T>();

        let uniform_buffer = Rc::new(VkBuffer::new(
            device,
            size as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            physical_device.get_mem_properties()
        ));

        let data = uniform_buffer.map() as *mut T;
        
        VkUniformBuffer {
            uniform_buffer: uniform_buffer,
            data: data,
            size: size
        }
    }

    pub fn track_buffer(&self) -> Rc<VkBuffer> {
        self.uniform_buffer.clone()
    }

    pub fn get_buffer(&self) -> vk::Buffer {
        self.uniform_buffer.get_buffer()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn data<T: ToAny>(&mut self) -> &mut T {
        let data = unsafe { self.data.as_mut().unwrap() };
        match data.as_any().downcast_mut::<T>() {
            Some(i) => i,
            None => panic!("Failed to get uniform buffer. (Generic type mismatch)")
        }
    }
}

impl Drop for VkUniformBuffer {
    fn drop(&mut self) {
        self.uniform_buffer.unmap();
    }
}