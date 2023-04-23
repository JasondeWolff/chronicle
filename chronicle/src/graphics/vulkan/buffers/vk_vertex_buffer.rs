use ash::vk;
use crate::graphics::*;

pub struct VkVertexBuffer {
    vertex_buffer: Arc<VkBuffer>,
    vertex_count: u32,
    vertex_stride: u32,
    dynamic: bool
}

impl VkVertexBuffer {
    pub fn new<T: Sized>(
        app: &mut VkApp,
        vertices: &Vec<T>,
        dynamic: bool
    ) -> Self {
        let stride = std::mem::size_of::<T>();
        let size = (stride * vertices.len()) as u64;
        
        let vertex_buffer = if dynamic {
            let vertex_buffer = VkBuffer::new(
                "Vertex buffer",
                app.get_device().clone(),
                app.get_allocator(),
                size,
                vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                None
            );
    
            unsafe {
                let data_ptr = vertex_buffer.map() as *mut T;
                data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
                vertex_buffer.unmap();
            }

            vertex_buffer
        } else {
            let staging_buffer = VkBuffer::new(
                "Vertex staging buffer",
                app.get_device().clone(),
                app.get_allocator(),
                size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                None
            );
    
            unsafe {
                let data_ptr = staging_buffer.map() as *mut T;
                data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
                staging_buffer.unmap();
            }
    
            let vertex_buffer = VkBuffer::new(
                "Vertex buffer",
                app.get_device().clone(),
                app.get_allocator(),
                size,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                None
            );
    
            let cmd_queue = app.get_cmd_queue();
            let mut cmd_queue = cmd_queue.as_mut();
            let cmd_buffer = cmd_queue.get_cmd_buffer(); {
                let cmd_buffer_ref = cmd_buffer.as_ref();
                cmd_buffer_ref.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
                cmd_buffer_ref.copy_buffers(&staging_buffer, &vertex_buffer);
                cmd_buffer_ref.end();
            }
            cmd_queue.submit_cmd_buffer(cmd_buffer, None, None);
            app.get_device().wait_idle();

            vertex_buffer
        };

        VkVertexBuffer {
            vertex_buffer: Arc::new(vertex_buffer),
            vertex_count: vertices.len() as u32,
            vertex_stride: stride as u32,
            dynamic: dynamic
        }
    }

    pub fn get_buffer(&self) -> Arc<VkBuffer> {
        self.vertex_buffer.clone()
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }

    pub fn vertex_stride(&self) -> u32 {
        self.vertex_stride
    }

    pub fn set_vertex_data<T: Sized>(&mut self, vertices: &Vec<T>) {
        assert!(self.dynamic, "Failed to set vertex data. (Not marked as dynamic)");
        assert!(vertices.len() < self.vertex_buffer.get_size() as usize, "Failed to set vertex data. (Exceeds available memory)");

        unsafe {
            let data_ptr = self.vertex_buffer.map() as *mut T;
            data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
            self.vertex_buffer.unmap();
        }

        self.vertex_count = vertices.len() as u32;
    }
}