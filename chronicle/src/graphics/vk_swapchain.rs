use ash::vk;
use ash::version::DeviceV1_0;

use std::rc::Rc;
use std::ptr;

use crate::graphics::*;
use utility::constants::MAX_FRAMES_IN_FLIGHT;

pub struct VkSwapchain {
    device: Rc<VkLogicalDevice>,

    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    _swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_imageviews: Vec<vk::ImageView>,

    framebuffers: Vec<vk::Framebuffer>,

    image_available_semaphores: Vec<Rc<VkSemaphore>>,
    render_finished_semaphores: Vec<Rc<VkSemaphore>>,
    inflight_fences: Vec<VkFence>,
    current_frame: usize
}

pub struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl VkSwapchain {
    pub fn new(
        instance: &VkInstance,
        device: Rc<VkLogicalDevice>,
        physical_device: &VkPhysicalDevice,
        width: u32, height: u32
    ) -> VkSwapchain {
        let swapchain_support = Self::query_swapchain_support(physical_device.get_device(), instance.get_surface_loader(), *instance.get_surface());

        let surface_format = Self::choose_swapchain_format(&swapchain_support.formats);
        let present_mode = Self::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = Self::choose_swapchain_extent(&swapchain_support.capabilities, width, height);

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let queue_family = device.get_queue_family_indices();
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
            surface: *instance.get_surface(),
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

        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance.get_instance(), device.get_device());
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
        let swapchain_imageviews = Self::create_image_views(
            &device.get_device(),
            surface_format.format,
            &swapchain_images,
        );

        let mut image_available_semaphores = Vec::new();
        let mut render_finished_semaphores = Vec::new();
        let mut inflight_fences = Vec::new();
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores.push(VkSemaphore::new(device.clone()));
            render_finished_semaphores.push(VkSemaphore::new(device.clone()));
            inflight_fences.push(VkFence::new(device.clone()));
        }

        VkSwapchain {
            device: device,

            swapchain_loader: swapchain_loader,
            swapchain: swapchain,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            _swapchain_images: swapchain_images,
            swapchain_imageviews: swapchain_imageviews,
            framebuffers: Vec::new(),
            image_available_semaphores: image_available_semaphores,
            render_finished_semaphores: render_finished_semaphores,
            inflight_fences: inflight_fences,
            current_frame: 0
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

    fn create_image_views(
        device: &ash::Device,
        surface_format: vk::Format,
        images: &Vec<vk::Image>,
    ) -> Vec<vk::ImageView> {
        let mut swapchain_imageviews = vec![];

        for &image in images.iter() {
            let imageview_create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::ImageViewCreateFlags::empty(),
                view_type: vk::ImageViewType::TYPE_2D,
                format: surface_format,
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

            let imageview = unsafe {
                device
                    .create_image_view(&imageview_create_info, None)
                    .expect("Failed to create Image View.")
            };
            swapchain_imageviews.push(imageview);
        }

        swapchain_imageviews
    }

    pub fn get_extent(&self) -> &vk::Extent2D {
        &self.swapchain_extent
    }

    pub fn get_format(&self) -> &vk::Format {
        &self.swapchain_format
    }

    pub fn build_framebuffers(&mut self, render_pass: &VkRenderPass) {
        self.framebuffers.clear();

        for &image_view in self.swapchain_imageviews.iter() {
            let attachments = [image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass: *render_pass.get_render_pass(),
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: self.swapchain_extent.width,
                height: self.swapchain_extent.height,
                layers: 1,
            };

            let framebuffer = unsafe {
                self.device.get_device()
                    .create_framebuffer(&framebuffer_create_info, None)
                    .expect("Failed to create Framebuffer!")
            };

            self.framebuffers.push(framebuffer);
        }
    }

    pub fn get_framebuffer(&self, idx: usize) -> &vk::Framebuffer {
        &self.framebuffers[idx]
    }

    pub fn get_framebuffer_count(&self) -> usize {
        self.framebuffers.len()
    }

    pub fn next_image(&self) -> (u32, &VkFence) {
        self.inflight_fences[self.current_frame].wait();

        let image_idx = unsafe {
            self.swapchain_loader
                .acquire_next_image(
                    self.swapchain,
                    std::u64::MAX,
                    *self.image_available_semaphores[self.current_frame].get_semaphore(),
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image.").0
        };

        self.inflight_fences[self.current_frame].reset();

        (image_idx, &self.inflight_fences[self.current_frame])
    }

    pub fn image_available_semaphore(&self) -> Rc<VkSemaphore> {
        self.image_available_semaphores[self.current_frame].clone()
    }

    pub fn render_finished_semaphore(&self) -> Rc<VkSemaphore> {
        self.render_finished_semaphores[self.current_frame].clone()
    }

    pub fn present(&mut self, image_idx: u32, wait_semaphores: &Vec<&VkSemaphore>) {
        let mut wait_semaphores_raw = Vec::new();
        for wait_semaphore in wait_semaphores {
            wait_semaphores_raw.push(*wait_semaphore.get_semaphore());
        }

        let swapchains = [self.swapchain];
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: wait_semaphores_raw.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_idx,
            p_results: ptr::null_mut(),
        };

        unsafe {
            self.swapchain_loader
                .queue_present(self.device.get_present_queue(), &present_info)
                .expect("Failed to execute queue present.");
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

impl Drop for VkSwapchain {
    fn drop(&mut self) {
        unsafe {
            for &imageview in self.swapchain_imageviews.iter() {
                self.device.get_device().destroy_image_view(imageview, None);
            }

            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
        }
    }
}