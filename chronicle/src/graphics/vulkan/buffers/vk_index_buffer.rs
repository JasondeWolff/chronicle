use ash::vk;

use crate::graphics::*;

pub struct VkIndexBuffer {
    index_buffer: VkBuffer,
    index_count: u32
}

impl VkIndexBuffer {
    pub fn new(
        app: RcCell<VkApp>,
        indices: &Vec<u32>
    ) -> Self {
        let mut app = app.as_mut();

        let size = (std::mem::size_of::<u32>() * indices.len()) as u64;
        
        let staging_buffer = VkBuffer::new(
            app.get_device().clone(),
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            app.get_physical_device().get_mem_properties()
        );

        unsafe {
            let data_ptr = staging_buffer.map() as *mut u32;
            data_ptr.copy_from_nonoverlapping(indices.as_ptr(), indices.len());
            staging_buffer.unmap();
        }

        let index_buffer = VkBuffer::new(
            app.get_device().clone(),
            size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            app.get_physical_device().get_mem_properties()
        );

        let cmd_queue = app.get_cmd_queue();
        let cmd_buffer = cmd_queue.get_cmd_buffer(); {
            let cmd_buffer_ref = cmd_buffer.as_ref();
            cmd_buffer_ref.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            cmd_buffer_ref.copy_buffers(&staging_buffer, &index_buffer);
            cmd_buffer_ref.end();
        }
        cmd_queue.submit_cmd_buffer(cmd_buffer, None, None);
        app.get_device().wait_idle();

        VkIndexBuffer {
            index_buffer: index_buffer,
            index_count: indices.len() as u32
        }
    }

    pub fn get_buffer(&self) -> vk::Buffer {
        self.index_buffer.get_buffer()
    }

    pub fn index_count(&self) -> u32 {
        self.index_count
    }
}