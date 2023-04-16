use ash::vk;

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
    depth_img: VkImage,

    framebuffers: Vec<vk::Framebuffer>,

    present_render_pass: Rc<VkRenderPass>,

    image_available_semaphores: Vec<Rc<VkSemaphore>>,
    render_finished_semaphores: Vec<Rc<VkSemaphore>>,
    inflight_fences: [Rc<VkFence>; MAX_FRAMES_IN_FLIGHT],
    current_frame: usize,
    current_img: u32
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
    ) -> RcCell<Self> {
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
            device.clone(),
            surface_format.format,
            &swapchain_images,
        );

        let mut image_available_semaphores = Vec::new();
        let mut render_finished_semaphores = Vec::new();
        let mut inflight_fences = Vec::new();
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores.push(VkSemaphore::new(device.clone()));
            render_finished_semaphores.push(VkSemaphore::new(device.clone()));
            inflight_fences.push(VkFence::new(device.clone(), true));
        }

        let mut depth_img = VkImage::new(
            device.clone(),
            width, height,
            1,
            Self::optimal_depth_format(instance, physical_device),
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            physical_device.get_mem_properties()
        );

        let mut framebuffers = Vec::new();
        let present_render_pass = VkRenderPass::new(
            device.clone(),
            surface_format.format,
            depth_img.format()
        );
        
        for &image_view in swapchain_imageviews.iter() {
            let attachments = [image_view, depth_img.get_image_view()];

            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass: present_render_pass.get_render_pass(),
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: extent.width,
                height: extent.height,
                layers: 1,
            };

            let framebuffer = unsafe {
                device.get_device()
                    .create_framebuffer(&framebuffer_create_info, None)
                    .expect("Failed to create Framebuffer!")
            };

            framebuffers.push(framebuffer);
        }

        RcCell::new(VkSwapchain {
            device: device,

            swapchain_loader: swapchain_loader,
            swapchain: swapchain,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            _swapchain_images: swapchain_images,
            swapchain_imageviews: swapchain_imageviews,
            depth_img: depth_img,

            framebuffers: framebuffers,

            present_render_pass: present_render_pass,
            image_available_semaphores: image_available_semaphores,
            render_finished_semaphores: render_finished_semaphores,
            inflight_fences: inflight_fences.try_into().unwrap_or_else(|_| panic!("")),

            current_frame: 0,
            current_img: 0
        })
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
        device: Rc<VkLogicalDevice>,
        surface_format: vk::Format,
        images: &Vec<vk::Image>,
    ) -> Vec<vk::ImageView> {
        let mut swapchain_imageviews = vec![];

        for &image in images.iter() {
            swapchain_imageviews.push(VkImage::create_image_view(
                device.clone(),
                image,
                surface_format
            ));
        }

        swapchain_imageviews
    }

    fn optimal_depth_format(instance: &VkInstance, physical_device: &VkPhysicalDevice) -> vk::Format {
        Self::find_supported_format(
            instance, physical_device,
            &[
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    fn find_supported_format(
        instance: &VkInstance, physical_device: &VkPhysicalDevice,
        candidate_formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags
    ) -> vk::Format {
        for &format in candidate_formats.iter() {
            let format_properties = unsafe {
                instance.get_instance()
                    .get_physical_device_format_properties(physical_device.get_device(), format)
            };

            if tiling == vk::ImageTiling::LINEAR && format_properties.linear_tiling_features.contains(features) {
                return format.clone();
            } else if tiling == vk::ImageTiling::OPTIMAL && format_properties.optimal_tiling_features.contains(features) {
                return format.clone();
            }
        }

        panic!("Failed to find supported format.")
    }

    pub fn get_extent(&self) -> &vk::Extent2D {
        &self.swapchain_extent
    }

    pub fn get_format(&self) -> vk::Format {
        self.swapchain_format
    }

    pub fn get_current_framebuffer(&self) -> &vk::Framebuffer {
        &self.framebuffers[self.current_img as usize]
    }

    pub fn get_framebuffer_count(&self) -> usize {
        self.framebuffers.len()
    }

    pub fn get_render_pass(&self) -> Rc<VkRenderPass> { // temporary!!!
        self.present_render_pass.clone()
    }

    pub fn next_image(&mut self) {
        self.inflight_fences[self.current_frame].wait();

        self.current_img = unsafe {
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
    }

    pub fn get_current_img(&self) -> u32 {
        self.current_img
    }

    pub fn image_available_semaphore(&self) -> Rc<VkSemaphore> {
        self.image_available_semaphores[self.current_frame].clone()
    }

    pub fn render_finished_semaphore(&self) -> Rc<VkSemaphore> {
        self.render_finished_semaphores[self.current_frame].clone()
    }

    pub fn present(&mut self, fence: Rc<VkFence>, wait_semaphores: &Vec<&VkSemaphore>) {
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
            p_image_indices: &self.current_img,
            p_results: ptr::null_mut(),
        };

        unsafe {
            self.swapchain_loader
                .queue_present(self.device.get_present_queue(), &present_info)
                .expect("Failed to execute queue present.");
        }

        self.inflight_fences[self.current_frame] = fence;
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