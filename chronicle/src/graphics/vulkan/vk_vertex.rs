use ash::vk;
use memoffset::offset_of;

use crate::resources::Vertex;

pub struct VkVertex;

pub trait VkVertexDescs {
    fn get_binding_desc() -> Vec<vk::VertexInputBindingDescription>;
    fn get_attribute_desc() -> Vec<vk::VertexInputAttributeDescription>;
}

impl VkVertexDescs for VkVertex {
    fn get_binding_desc() -> Vec<vk::VertexInputBindingDescription> {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }].to_vec()
    }

    fn get_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, normal) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, tangent) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 3,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, tex_coord) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 4,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, tex_coord_1) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 5,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            }
        ].to_vec()
    }
}