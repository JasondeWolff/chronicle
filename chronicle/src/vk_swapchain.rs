use ash::vk;
use crate::vk_device::QueueFamilyIndices;

pub struct VkSwapchain {
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
}

pub struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl VkSwapchain {
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
        queue_family: &QueueFamilyIndices,
        width: u32, height: u32
    ) -> Self {
        let swapchain_support = Self::query_swapchain_support(physical_device, surface_loader, surface);

        let surface_format = Self::choose_swapchain_format(&swapchain_support.formats);
        let present_mode = Self::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = Self::choose_swapchain_extent(&swapchain_support.capabilities, 1280, 720);

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
            if queue_family.graphics_family != queue_family.present_family {
                (
                    vk::SharingMode::CONCURRENT,
                    2,
                    vec![
                        queue_family.graphics_family.unwrap(),
                        queue_family.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, vec![])
            };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: std::ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: surface,
            min_image_count: image_count,
            image_color_space: surface_format.color_space,
            image_format: surface_format.format,
            image_extent: extent,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode,
            p_queue_family_indices: queue_family_indices.as_ptr(),
            queue_family_index_count,
            pre_transform: swapchain_support.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            image_array_layers: 1,
        };

        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Swapchain!")
        };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get Swapchain Images.")
        };

        VkSwapchain {
            swapchain_loader: swapchain_loader,
            swapchain: swapchain,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            swapchain_images: swapchain_images,
        }
    }

    pub fn query_swapchain_support(
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> SwapChainSupportDetail {
        unsafe {
            let capabilities = surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .expect("Failed to query for surface capabilities.");
            let formats = surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .expect("Failed to query for surface formats.");
            let present_modes = surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .expect("Failed to query for surface present mode.");

            SwapChainSupportDetail {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    fn choose_swapchain_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }

        return available_formats.first().unwrap().clone();
    }

    fn choose_swapchain_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        for &available_present_mode in available_present_modes.iter() {
            if available_present_mode == vk::PresentModeKHR::MAILBOX {
                return available_present_mode;
            }
        }

        vk::PresentModeKHR::FIFO
    }

    fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR, width: u32, height: u32) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::max_value() {
            capabilities.current_extent
        } else {
            use num::clamp;

            vk::Extent2D {
                width: clamp(
                    width,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: clamp(
                    height,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }
}