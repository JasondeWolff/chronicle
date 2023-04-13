use crate::graphics::*;
use crate::resources::Vertex;

pub struct VkMesh {
    vertex_buffer: VkVertexBuffer,
    index_buffer: VkIndexBuffer
}

impl VkMesh {
    pub fn new(
        device: Rc<VkLogicalDevice>,
        physical_device: &VkPhysicalDevice,
        cmd_pool: Rc<VkCmdPool>,
        vertices: &Vec<Vertex>,
        indices: &Vec<u32>
    ) -> Self {
        VkMesh {
            vertex_buffer: VkVertexBuffer::new(device.clone(), physical_device, cmd_pool.clone(), vertices),
            index_buffer: VkIndexBuffer::new(device, physical_device, cmd_pool, indices)
        }
    }

    pub fn draw_cmds(&self, cmd_buffer: &VkCmdBuffer) {
        cmd_buffer.bind_vertex_buffer(&self.vertex_buffer);
        cmd_buffer.bind_index_buffer(&self.index_buffer);
        cmd_buffer.draw_indexed(self.index_buffer.index_count(), 1, 0, 0, 0);
    }
}