use ash::vk;
use cgmath::SquareMatrix;

use crate::graphics::*;

pub struct VkBlasInstance {
    instance: vk::AccelerationStructureInstanceKHR
}

impl VkBlasInstance {
    pub fn new(
        transform_matrix: Matrix4<f32>,
        blas: ArcMutex<VkBlas>,
        custom_idx: u32,
        mask: u8
    ) -> Self {
        let instance = vk::AccelerationStructureInstanceKHR {
            transform: mat4_to_khr_transform_matrix(transform_matrix),
            instance_custom_index_and_mask: vk::Packed24_8::new(custom_idx, mask),
            instance_shader_binding_table_record_offset_and_flags: vk::Packed24_8::new(0, 0),
            acceleration_structure_reference: blas.as_ref().get_accel_ref()
        };

        VkBlasInstance {
            instance
        }
    }
    
    pub fn get_instance(&self) -> &vk::AccelerationStructureInstanceKHR {
        &self.instance
    }
}

fn mat4_to_khr_transform_matrix(mut m: Matrix4<f32>) -> vk::TransformMatrixKHR {
    m.transpose_self();
    
    let mut matrix = [0.0f32; 12];
    unsafe {
        let elems: [f32; 16] = std::mem::transmute(m);
        std::ptr::copy_nonoverlapping(&elems as *const f32, &mut matrix as *mut f32, 12);
    }

    vk::TransformMatrixKHR {
        matrix: matrix
    }
}