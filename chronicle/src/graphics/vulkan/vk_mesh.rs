use crate::graphics::*;
use crate::resources::Vertex;

pub struct VkMesh {
    vertex_buffer: VkVertexBuffer,
    index_buffer: VkIndexBuffer
}

impl VkMesh {
    pub fn new(
        app: &mut VkApp,
        vertices: &Vec<Vertex>,
        indices: &Vec<u32>
    ) -> Self {
        VkMesh {
            vertex_buffer: VkVertexBuffer::new(app, vertices, false),
            index_buffer: VkIndexBuffer::new(app, indices, false)
        }
    }

    pub fn draw_cmds(&self, cmd_buffer: &mut VkCmdBuffer) {
        cmd_buffer.bind_vertex_buffer(&self.vertex_buffer);
        cmd_buffer.bind_index_buffer(&self.index_buffer);
        cmd_buffer.draw_indexed(self.index_buffer.index_count(), 1, 0, 0, 0);
    }
}