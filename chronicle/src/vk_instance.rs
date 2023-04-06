use ash::version::EntryV1_0;
use ash::{vk, version::InstanceV1_0};
use crate::utility::constants::{ENGINE_TITLE, ENGINE_VERSION, APPLICATION_VERSION, API_VERSION, VALIDATION};
use crate::utility;

use std::ffi::CString;
use std::ptr;
use std::os::raw::c_void;

use crate::vk_device::{VkPhysicalDevice, VkLogicalDevice};
use crate::vk_swapchain::VkSwapchain;
use crate::window::Window;

pub struct VkInstance {
    _entry: ash::Entry,
    instance: ash::Instance,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    surface_loader: ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,

    physical_device: VkPhysicalDevice,
    device: VkLogicalDevice,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain: VkSwapchain
}

impl VkInstance {
    pub fn new(title: &'static str, window: &Window) -> VkInstance {
        let entry = ash::Entry::new().unwrap();
        let instance = Self::create_instance(&entry, title);
        let (debug_utils_loader, debug_messenger) = utility::debug::setup_debug_utils(true, &entry, &instance);
        let surface = unsafe { utility::platforms::create_surface(&entry, &instance, &window.get_winit_window()).expect("Failed to create a surface.") };
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);

        let physical_device = VkPhysicalDevice::new(&instance, &surface_loader, surface);
        let logical_device = VkLogicalDevice::new(&instance, &physical_device, &surface_loader, surface);
        let graphics_queue = logical_device.get_graphics_queue();
        let present_queue = logical_device.get_present_queue();

        let swapchain = VkSwapchain::new(
            &instance,
            &logical_device.get_device(), physical_device.get_device(),
            &surface_loader, surface,
            logical_device.get_queue_family_indices(),
            window.width(),
            window.height()
        );

        VkInstance {
            _entry: entry,
            instance: instance,
            debug_utils_loader: debug_utils_loader,
            debug_messenger: debug_messenger,
            surface_loader: surface_loader,
            surface: surface,

            physical_device: physical_device,
            device: logical_device,
            graphics_queue: graphics_queue,
            present_queue: present_queue,

            swapchain: swapchain
        }
    }

    fn create_instance(entry: &ash::Entry, title: &'static str) -> ash::Instance {
        if !(VALIDATION.is_enable && utility::debug::check_validation_layer_support(entry, &VALIDATION.required_validation_layers.to_vec())) {
            panic!("Failed to enable validation layers.");
        }

        let app_name = CString::new(title).unwrap();
        let engine_name = CString::new(ENGINE_TITLE).unwrap();
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: APPLICATION_VERSION,
            p_engine_name: engine_name.as_ptr(),
            engine_version: ENGINE_VERSION,
            api_version: API_VERSION,
        };

        let debug_utils_create_info = utility::debug::populate_debug_messenger_create_info();
        let extension_names = utility::platforms::required_extension_names();

        let required_validation_layer_raw_names: Vec<CString> = VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let enable_layer_names: Vec<*const i8> = required_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let debug_utils_ptr = if VALIDATION.is_enable {
            &debug_utils_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void
        } else {
            ptr::null()
        };

        let enabled_layer_names_ptrptr = if VALIDATION.is_enable {
            enable_layer_names.as_ptr()
        } else {
            ptr::null()
        };

        let enabled_layer_count = if VALIDATION.is_enable {
            enable_layer_names.len()
        } else {
            0
        } as u32;

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: debug_utils_ptr,
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: enabled_layer_names_ptrptr,
            enabled_layer_count: enabled_layer_count,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
        };

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance.")
        };

        instance
    }
}

impl Drop for VkInstance {
    fn drop(&mut self) {
        unsafe {
            if VALIDATION.is_enable {
                self.debug_utils_loader.destroy_debug_utils_messenger(self.debug_messenger, None);
            }

            self.instance.destroy_instance(None);
        }
    }
}