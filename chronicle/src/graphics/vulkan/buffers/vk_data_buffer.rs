use ash::vk;

use crate::graphics::*;

pub struct VkDataBuffer<T> {
    buffer: Arc<VkBuffer>,
    count: u32,
    stride: u32,
    dynamic: bool,
    data: *mut T
}

impl<T> VkDataBuffer<T> {
    pub fn new(
        name: &'static str,
        app: &mut VkApp,
        data: &Vec<T>,
        usage: vk::BufferUsageFlags,
        mem_properties: vk::MemoryPropertyFlags,
        dynamic: bool
    ) -> Self {
        let stride = std::mem::size_of::<T>();
        let size: u64 = (stride * data.len()) as u64;

        let buffer = if dynamic {
            let buffer = VkBuffer::new(
                format!("{name} BUFFER"),
                app.get_device().clone(),
                app.get_allocator(),
                size,
                usage,
                mem_properties,
                None
            );
    
            unsafe {
                let data_ptr = buffer.map() as *mut T;
                data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
                buffer.unmap();
            }

            buffer
        } else {
            let staging_buffer = VkBuffer::new(
                format!("{name} STAGING BUFFER"),
                app.get_device().clone(),
                app.get_allocator(),
                size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                None
            );

            unsafe {
                let data_ptr = staging_buffer.map() as *mut T;
                data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
                staging_buffer.unmap();
            }

            let buffer = VkBuffer::new(
                format!("{name} BUFFER"),
                app.get_device().clone(),
                app.get_allocator(),
                size,
                vk::BufferUsageFlags::TRANSFER_DST | usage,
                mem_properties,
                None
            );

            let cmd_queue = app.get_cmd_queue();
            let mut cmd_queue = cmd_queue.as_mut();
            let cmd_buffer = cmd_queue.get_cmd_buffer(); {
                let cmd_buffer_ref = cmd_buffer.as_ref();
                cmd_buffer_ref.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
                cmd_buffer_ref.copy_buffers(&staging_buffer, &buffer);
                cmd_buffer_ref.end();
            }
            cmd_queue.submit_cmd_buffer(cmd_buffer, None, None);
            app.get_device().wait_idle();

            buffer
        };

        let data_ptr = if dynamic {
            buffer.map() as *mut T
        } else {
            std::ptr::null_mut()
        };

        VkDataBuffer {
            buffer: Arc::new(buffer),
            count: data.len() as u32,
            stride: stride as u32,
            dynamic: dynamic,
            data: data_ptr
        }
    }

    pub fn get_buffer(&self) -> Arc<VkBuffer> {
        self.buffer.clone()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn get_stride(&self) -> u32 {
        self.stride
    }

    pub fn set_data(&mut self, data: &Vec<T>) {
        assert!(self.dynamic, "Failed to set index data. (Not marked as dynamic)");
        assert!(data.len() * std::mem::size_of::<T>() < self.buffer.get_size() as usize, "Failed to set index data. (Exceeds available memory)");

        unsafe {
            self.data.copy_from_nonoverlapping(data.as_ptr(), data.len());
        }

        self.count = data.len() as u32;
    }

    pub fn get_data_ptr(&self) -> *mut T {
        self.data
    }
}