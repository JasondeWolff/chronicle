use std::rc::Rc;

use ash::vk;

use crate::app;
use crate::graphics::*;
use crate::resources::Vertex;

pub struct VkVertexBuffer {
    vertex_buffer: VkBuffer,
    upload_cmd_buffer: Vec<VkCmdBuffer>,
    vertex_count: u32
}

use cgmath::*;
const VERTEX_DATA: [Vertex; 4] = [
    Vertex {
        position: Vector3::new(-0.5, -0.5, 0.0),
        normal: Vector3::new(0.0, 0.0, 0.0),
        tangent: Vector4::new(0.0, 0.0, 0.0, 0.0),
        tex_coord: Vector2::new(0.0, 0.0),
        tex_coord_1: Vector2::new(0.0, 0.0),
        color: Vector4::new(1.0, 0.0, 0.0, 1.0)
    },
    Vertex {
        position: Vector3::new(0.5, -0.5, 0.0),
        normal: Vector3::new(0.0, 0.0, 0.0),
        tangent: Vector4::new(0.0, 0.0, 0.0, 0.0),
        tex_coord: Vector2::new(0.0, 0.0),
        tex_coord_1: Vector2::new(0.0, 0.0),
        color: Vector4::new(0.0, 1.0, 0.0, 1.0)
    },
    Vertex {
        position:  Vector3::new(0.5, 0.5, 0.0),
        normal: Vector3::new(0.0, 0.0, 0.0),
        tangent: Vector4::new(0.0, 0.0, 0.0, 0.0),
        tex_coord: Vector2::new(0.0, 0.0),
        tex_coord_1: Vector2::new(0.0, 0.0),
        color: Vector4::new(0.0, 0.0, 1.0, 1.0)
    },
    Vertex {
        position:  Vector3::new(-0.5, 0.5, 0.0),
        normal: Vector3::new(0.0, 0.0, 0.0),
        tangent: Vector4::new(0.0, 0.0, 0.0, 0.0),
        tex_coord: Vector2::new(0.0, 0.0),
        tex_coord_1: Vector2::new(0.0, 0.0),
        color: Vector4::new(1.0, 1.0, 1.0, 1.0)
    }
];

impl VkVertexBuffer {
    pub fn new(
        device: Rc<VkLogicalDevice>,
        physical_device: &VkPhysicalDevice,
        cmd_pool: Rc<VkCmdPool>,
        vertices: &Vec<Vertex>
    ) -> Self {
        //let vertices = vec![VERTEX_DATA[0].clone(), VERTEX_DATA[1].clone(), VERTEX_DATA[2].clone(), VERTEX_DATA[3].clone()];

        let size = (std::mem::size_of::<Vertex>() * vertices.len()) as u64;
        
        let staging_buffer = VkBuffer::new(
            device.clone(),
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            physical_device.get_mem_properties()
        );

        unsafe {
            let data_ptr = staging_buffer.map() as *mut Vertex;
            data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
            staging_buffer.unmap();
        }

        let vertex_buffer = VkBuffer::new(
            device.clone(),
            size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            physical_device.get_mem_properties()
        );

        let cmd_buffers = VkCmdBuffer::new(device, cmd_pool, 1);
        let cmd_buffer = &cmd_buffers[0];
        cmd_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        cmd_buffer.copy_buffers(&staging_buffer, &vertex_buffer);
        cmd_buffer.end();
        cmd_buffer.submit(None, None, None);
        app().graphics().wait_idle();

        VkVertexBuffer {
            vertex_buffer: vertex_buffer,
            upload_cmd_buffer: cmd_buffers,
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