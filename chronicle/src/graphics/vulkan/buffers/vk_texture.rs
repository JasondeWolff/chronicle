use ash::vk;

use crate::graphics::*;
use crate::resources::Texture;

pub struct VkTexture {
    image: VkImage,
    mip_levels: u32
}

impl VkTexture {
    pub fn new(
        app: RcCell<VkApp>,
        texture_resource: Resource<Texture>
    ) -> Self {
        let mut app = app.as_mut();

        let (image_width, image_height, channel_count) = (texture_resource.as_ref().width, texture_resource.as_ref().height, texture_resource.as_ref().channel_count);
        assert_eq!(channel_count, 4, "Failed to create new image.");
        
        let image_size = (std::mem::size_of::<u8>() as u32 * image_width * image_height * channel_count) as vk::DeviceSize;
        let image_data = &texture_resource.as_ref().data;
        let mip_levels = texture_resource.as_ref().mip_levels;

        let staging_buffer = VkBuffer::new(
            app.get_device().clone(),
            app.get_allocator(),
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            app.get_physical_device().get_mem_properties()
        );

        unsafe {
            let data_ptr = staging_buffer.map() as *mut u8;
            data_ptr.copy_from_nonoverlapping(image_data.as_ptr(), image_data.len());
            staging_buffer.unmap();
        }

        let image = VkImage::new(
            app.get_device().clone(),
            image_width, image_height,
            mip_levels,
            vk::Format::R8G8B8A8_UNORM,
            vk::SampleCountFlags::TYPE_1,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            app.get_physical_device().get_mem_properties(),
        );

        let cmd_queue = app.get_cmd_queue();
        let cmd_buffer = cmd_queue.get_cmd_buffer(); {
            let cmd_buffer_ref = cmd_buffer.as_ref();
            cmd_buffer_ref.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            cmd_buffer_ref.transition_image_layout(
                &image,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                mip_levels
            );
            cmd_buffer_ref.copy_buffer_to_image(&staging_buffer, &image);
            cmd_buffer_ref.generate_mips(&image, mip_levels);
            cmd_buffer_ref.end();
        }
        cmd_queue.submit_cmd_buffer(cmd_buffer, None, None);
        app.get_device().wait_idle();

        VkTexture {
            image: image,
            mip_levels: mip_levels
        }
    }

    pub fn get_image(&self) -> &VkImage {
        &self.image
    }

    pub fn get_image_view(&mut self) -> vk::ImageView {
        self.image.get_image_view()
    }

    pub fn mip_levels(&self) -> u32 {
        self.mip_levels
    }
}