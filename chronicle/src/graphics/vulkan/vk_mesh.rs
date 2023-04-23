use crate::graphics::*;
use crate::resources::Vertex;

pub struct VkMesh {
    vertex_buffer: VkVertexBuffer,
    index_buffer: VkIndexBuffer,
    blas: ArcMutex<VkBlas>
}

impl VkMesh {
    pub fn new(
        app: &mut VkApp,
        vertices: &Vec<Vertex>,
        indices: &Vec<u32>
    ) -> Self {
        let vertex_buffer = VkVertexBuffer::new(app, vertices, false);
        let index_buffer = VkIndexBuffer::new(app, indices, false);

        let blas = VkBlas::new(
            &vertex_buffer,
            &index_buffer,
            vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_BUILD
        );
        VkBlas::build(app, &vec![blas.clone()], vk::AccelerationStructureBuildTypeKHR::DEVICE);

        VkMesh {
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            blas: blas
        }
    }

    pub fn draw_cmds(&self, cmd_buffer: &mut VkCmdBuffer) {
        cmd_buffer.bind_vertex_buffer(&self.vertex_buffer);
        cmd_buffer.bind_index_buffer(&self.index_buffer);
        cmd_buffer.draw_indexed(self.index_buffer.index_count(), 1, 0, 0, 0);
    }

    pub fn get_blas(&self) -> ArcMutex<VkBlas> {
        self.blas.clone()
    }
}