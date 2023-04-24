use crate::graphics::*;
use crate::resources::Vertex;

pub struct VkMesh {
    vertex_buffer: VkDataBuffer<Vertex>,
    index_buffer: VkDataBuffer<u32>,
    blas: ArcMutex<VkBlas>
}

impl VkMesh {
    pub fn new(
        app: &mut VkApp,
        vertices: &Vec<Vertex>,
        indices: &Vec<u32>
    ) -> Self {
        let vertex_buffer = VkDataBuffer::new(
            "Mesh Vertices",
            app,
            &vertices,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR | vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            false
        );
        let index_buffer = VkDataBuffer::new(
            "Mesh Indices",
            app,
            &indices,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR | vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            false
        );

        let blas = VkBlas::new(
            &vertex_buffer,
            &index_buffer,
            vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_BUILD | vk::BuildAccelerationStructureFlagsKHR::ALLOW_COMPACTION
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
        cmd_buffer.draw_indexed(self.index_buffer.get_count(), 1, 0, 0, 0);
    }

    pub fn get_blas(&self) -> ArcMutex<VkBlas> {
        self.blas.clone()
    }
}