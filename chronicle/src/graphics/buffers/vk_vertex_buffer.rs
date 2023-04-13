use std::rc::Rc;

use ash::vk;

use crate::graphics::*;
use crate::resources::Vertex;

pub struct VkVertexBuffer {
    vertex_buffer: VkBuffer,
    vertex_count: u32
}

use cgmath::*;
const VERTEX_DATA: [Vertex; 3] = [
    Vertex {
        position: Vector3::new(0.0, -0.5, 0.0),
        normal: Vector3::new(0.0, 0.0, 0.0),
        tangent: Vector4::new(0.0, 0.0, 0.0, 0.0),
        tex_coord: Vector2::new(0.0, 0.0),
        tex_coord_1: Vector2::new(0.0, 0.0),
        color: Vector4::new(1.0, 1.0, 1.0, 1.0)
    },
    Vertex {
        position: Vector3::new(0.5, 0.5, 0.0),
        normal: Vector3::new(0.0, 0.0, 0.0),
        tangent: Vector4::new(0.0, 0.0, 0.0, 0.0),
        tex_coord: Vector2::new(0.0, 0.0),
        tex_coord_1: Vector2::new(0.0, 0.0),
        color: Vector4::new(0.0, 1.0, 0.0, 1.0)
    },
    Vertex {
        position:  Vector3::new(-0.5, 0.5, 0.0),
        normal: Vector3::new(0.0, 0.0, 0.0),
        tangent: Vector4::new(0.0, 0.0, 0.0, 0.0),
        tex_coord: Vector2::new(0.0, 0.0),
        tex_coord_1: Vector2::new(0.0, 0.0),
        color: Vector4::new(0.0, 0.0, 1.0, 1.0)
    }
];

impl VkVertexBuffer {
    pub fn new(
        device: Rc<VkLogicalDevice>,
        physical_device: &VkPhysicalDevice,
        vertices: &Vec<Vertex>
    ) -> Self {
        let vertices = vec![VERTEX_DATA[0].clone(), VERTEX_DATA[1].clone(), VERTEX_DATA[2].clone()];

        let required_memory_flags: vk::MemoryPropertyFlags = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let vertex_buffer = VkBuffer::new(
            device,
            (std::mem::size_of::<Vertex>() * vertices.len()) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            required_memory_flags,
            physical_device.get_mem_properties()
        );

        unsafe {
            let data_ptr = vertex_buffer.map() as *mut Vertex;
            data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
            vertex_buffer.unmap();
        }
        
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