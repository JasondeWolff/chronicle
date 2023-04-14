use std::rc::Rc;

use ash::vk;

use crate::app;
use crate::graphics::*;
use crate::resources::Texture;

pub struct VkImage {
    device: Rc<VkLogicalDevice>,
    image: vk::Image,
    image_view: Option<vk::ImageView>,
    memory: vk::DeviceMemory,
    width: u32, height: u32,
    format: vk::Format
}

impl VkImage {
    pub fn new(
        device: Rc<VkLogicalDevice>,
        width: u32, height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        required_memory_properties: vk::MemoryPropertyFlags,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties
    ) -> Self {
        let image_create_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
            initial_layout: vk::ImageLayout::UNDEFINED,
        };

        let texture_image = unsafe {
            device.get_device()
                .create_image(&image_create_info, None)
                .expect("Failed to create Texture Image!")
        };

        let image_memory_requirement = unsafe {
            device.get_device()
                .get_image_memory_requirements(texture_image)
            };
        let memory_allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            allocation_size: image_memory_requirement.size,
            memory_type_index: VkBuffer::find_memory_type(
                image_memory_requirement.memory_type_bits,
                required_memory_properties,
                *device_memory_properties,
            ),
        };

        let texture_image_memory = unsafe {
            device.get_device()
                .allocate_memory(&memory_allocate_info, None)
                .expect("Failed to allocate Texture Image memory.")
        };

        unsafe {
            device.get_device()
                .bind_image_memory(texture_image, texture_image_memory, 0)
                .expect("Failed to bind Image Memmory.");
        }

        VkImage {
            device: device,
            image: texture_image,
            image_view: None,
            memory: texture_image_memory,
            width: width,
            height: height,
            format: format
        }
    }

    pub fn create_image_view(
        device: Rc<VkLogicalDevice>,
        image: vk::Image,
        format: vk::Format
    ) -> vk::ImageView {
        let imageview_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            image,
        };

        unsafe {
            device.get_device()
                .create_image_view(&imageview_create_info, None)
                .expect("Failed to create Image View.")
        }
    }

    pub fn get_image(&self) -> vk::Image {
        self.image
    }

    pub fn get_image_view(&mut self) -> vk::ImageView {
        match self.image_view {
            Some(image_view) => image_view,
            None => {
                self.image_view = Some(Self::create_image_view(
                    self.device.clone(),
                    self.image,
                    self.format
                ));
                self.image_view.unwrap()
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Drop for VkImage {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .free_memory(self.memory, None);

            self.device.get_device()
                .destroy_image(self.image, None);
        }
    }
}