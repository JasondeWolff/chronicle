use ash::vk;

use crate::graphics::*;

pub struct VkIndexBuffer {
    index_buffer: Arc<VkBuffer>,
    index_count: u32,
    dynamic: bool
}

impl VkIndexBuffer {
    pub fn new(
        app: &mut VkApp,
        indices: &Vec<u32>,
        dynamic: bool
    ) -> Self {
        //let mut app = app.as_mut();
        let size: u64 = (std::mem::size_of::<u32>() * indices.len()) as u64;

        let index_buffer = if dynamic {
            let index_buffer = VkBuffer::new(
                app.get_device().clone(),
                app.get_allocator(),
                size,
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                app.get_physical_device().get_mem_properties()
            );
    
            unsafe {
                let data_ptr = index_buffer.map() as *mut u32;
                data_ptr.copy_from_nonoverlapping(indices.as_ptr(), indices.len());
                index_buffer.unmap();
            }

            index_buffer
        } else {
            let staging_buffer = VkBuffer::new(
                app.get_device().clone(),
                app.get_allocator(),
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
                app.get_allocator(),
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

            index_buffer
        };

        VkIndexBuffer {
            index_buffer: Arc::new(index_buffer),
            index_count: indices.len() as u32,
            dynamic: dynamic
        }
    }

    pub fn track_buffer(&self) -> Arc<VkBuffer> {
        self.index_buffer.clone()
    }

    pub fn get_buffer(&self) -> vk::Buffer {
        self.index_buffer.get_buffer()
    }

    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    pub fn set_index_data(&mut self, indices: &Vec<u32>) {
        assert!(self.dynamic, "Failed to set index data. (Not marked as dynamic)");
        assert!(indices.len() < self.index_buffer.get_size() as usize, "Failed to set index data. (Exceeds available memory)");

        unsafe {
            let data_ptr = self.index_buffer.map() as *mut u32;
            data_ptr.copy_from_nonoverlapping(indices.as_ptr(), indices.len());
            self.index_buffer.unmap();
        }

        self.index_count = indices.len() as u32;
    }
}