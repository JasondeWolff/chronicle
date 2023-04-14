use std::rc::Rc;

use ash::vk;

use crate::app;
use crate::graphics::*;
use crate::resources::Texture;

pub struct VkTexture {
    image: VkImage
}

impl VkTexture {
    pub fn new(
        device: Rc<VkLogicalDevice>,
        physical_device: &VkPhysicalDevice,
        cmd_pool: Rc<VkCmdPool>,
        texture_resource: Resource<Texture>
    ) -> Self {
        let (image_width, image_height, channel_count) = (texture_resource.as_ref().width, texture_resource.as_ref().height, texture_resource.as_ref().channel_count);
        assert_eq!(channel_count, 3, "Failed to create new image.");
        
        let image_size = (std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;
        let image_data = &texture_resource.as_ref().data;

        let staging_buffer = VkBuffer::new(
            device.clone(),
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            physical_device.get_mem_properties()
        );

        unsafe {
            let data_ptr = staging_buffer.map() as *mut u8;
            data_ptr.copy_from_nonoverlapping(image_data.as_ptr(), image_data.len());
            staging_buffer.unmap();
        }

        let image = VkImage::new(
            device.clone(),
            image_width, image_height,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            physical_device.get_mem_properties(),
        );

        let cmd_buffers = VkCmdBuffer::new(device, cmd_pool, 1);
        let cmd_buffer = &cmd_buffers[0];
        cmd_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT); {
            cmd_buffer.transition_image_layout(
                &image,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL
            );
            cmd_buffer.copy_buffer_to_image(&staging_buffer, &image);
            cmd_buffer.transition_image_layout(
                &image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            );
        } cmd_buffer.end();
        cmd_buffer.submit(None, None, None);
        app().graphics().wait_idle();

        VkTexture {
            image: image
        }
    }
}