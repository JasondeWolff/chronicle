use std::rc::Rc;

use ash::vk;

use crate::graphics::*;

pub struct VkDescriptorSet {
    descriptor_set: vk::DescriptorSet
}

impl VkDescriptorSet {
    pub fn new<T: Default>(
        device: Rc<VkLogicalDevice>,
        desc_pool: &mut VkDescriptorPool,
        desc_layout: &VkDescriptorLayout,
        uniform_buffers: &Vec<VkUniformBuffer<T>>,
        // texture: &VkTexture,
        // texture_sampler: &VkSampler,
    ) -> Vec<Self> {
        let count = uniform_buffers.len();

        let mut layouts = Vec::new();
        for _ in 0..count {
            layouts.push(desc_layout.get_desc_layout());
        }
        
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            descriptor_pool: desc_pool.get_desc_pool(vk::DescriptorType::UNIFORM_BUFFER),
            descriptor_set_count: count as u32,
            p_set_layouts: layouts.as_ptr(),
        };

        let descriptor_sets = unsafe {
            device.get_device()
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets.")
        };

        // let descriptor_image_infos = [vk::DescriptorImageInfo {
        //     sampler: texture_sampler.get_sampler(),
        //     image_view: texture.get_image().get_image_view(),
        //     image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        // }];

        for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
            let descriptor_buffer_info = [vk::DescriptorBufferInfo {
                buffer: uniform_buffers[i].get_buffer(),
                offset: 0,
                range: std::mem::size_of::<T>() as u64,
            }];

            let descriptor_write_sets = [
                vk::WriteDescriptorSet {
                    s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                    p_next: std::ptr::null(),
                    dst_set: descriptor_set,
                    dst_binding: 0,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    p_image_info: std::ptr::null(),
                    p_buffer_info: descriptor_buffer_info.as_ptr(),
                    p_texel_buffer_view: std::ptr::null(),
                },
                // vk::WriteDescriptorSet {
                //     s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                //     p_next: std::ptr::null(),
                //     dst_set: descriptor_set,
                //     dst_binding: 1,
                //     dst_array_element: 0,
                //     descriptor_count: 1,
                //     descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                //     p_image_info: descriptor_image_infos.as_ptr(),
                //     p_buffer_info: std::ptr::null(),
                //     p_texel_buffer_view: std::ptr::null(),
                // },
            ];

            unsafe {
                device.get_device()
                    .update_descriptor_sets(&descriptor_write_sets, &[]);
            }
        }

        let mut descriptors = Vec::new();
        for descriptor_set in descriptor_sets.iter() {
            descriptors.push(VkDescriptorSet {
                descriptor_set: *descriptor_set
            });
        }
        descriptors
    }

    pub fn get_desc_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }
}