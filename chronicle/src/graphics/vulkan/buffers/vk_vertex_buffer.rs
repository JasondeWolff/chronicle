use ash::vk;
use crate::graphics::*;
use crate::resources::Vertex;

pub struct VkVertexBuffer {
    vertex_buffer: VkBuffer,
    vertex_count: u32
}

impl VkVertexBuffer {
    pub fn new(
        app: RcCell<VkApp>,
        vertices: &Vec<Vertex>
    ) -> Self {
        let mut app = app.as_mut();
        let size = (std::mem::size_of::<Vertex>() * vertices.len()) as u64;
        
        let staging_buffer = VkBuffer::new(
            app.get_device().clone(),
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            app.get_physical_device().get_mem_properties()
        );

        unsafe {
            let data_ptr = staging_buffer.map() as *mut Vertex;
            data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
            staging_buffer.unmap();
        }

        let vertex_buffer = VkBuffer::new(
            app.get_device().clone(),
            size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            app.get_physical_device().get_mem_properties()
        );

        let cmd_queue = app.get_cmd_queue();
        let cmd_buffer = cmd_queue.get_cmd_buffer(); {
            let cmd_buffer_ref = cmd_buffer.as_ref();
            cmd_buffer_ref.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            cmd_buffer_ref.copy_buffers(&staging_buffer, &vertex_buffer);
            cmd_buffer_ref.end();
        }
        cmd_queue.submit_cmd_buffer(cmd_buffer, None, None);
        app.get_device().wait_idle();

        VkVertexBuffer {
            vertex_buffer: vertex_buffer,
            vertex_count: vertices.len() as u32
        }
    }

    pub fn get_buffer(&self) -> vk::Buffer {
        self.vertex_buffer.get_buffer()
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }
}