use crate::graphics::*;
use crate::resources::Vertex;

pub struct VkMesh {
    vertex_buffer: VkVertexBuffer
}

impl VkMesh {
    pub fn new(
        device: Rc<VkLogicalDevice>,
        physical_device: &VkPhysicalDevice,
        vertices: &Vec<Vertex>
    ) -> Self {
        VkMesh {
            vertex_buffer: VkVertexBuffer::new(device, physical_device, vertices)
        }
    }

    pub fn draw_cmds(&self, cmd_buffer: &VkCmdBuffer) {
        cmd_buffer.bind_vertex_buffer(&self.vertex_buffer);
        cmd_buffer.draw(self.vertex_buffer.vertex_count(), 1, 0, 0);
    }
}