use ash::version::DeviceV1_0;
use ash::{vk, version::InstanceV1_0};

use crate::graphics::*;
use utility::constants::VALIDATION;

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use std::rc::Rc;

const DEVICE_EXTENSIONS: [&'static str; 1] = [
    "VK_KHR_swapchain"
];

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct VkPhysicalDevice {
    device: vk::PhysicalDevice,
    mem_properties: vk::PhysicalDeviceMemoryProperties
}

pub struct VkLogicalDevice {
    device: ash::Device,
    queue_indices: QueueFamilyIndices
}

impl VkPhysicalDevice {
    pub fn new(instance: &VkInstance) -> Self {
        let device = Self::pick_physical_device(
            instance.get_instance(),
            instance.get_surface_loader(),
            *instance.get_surface()
        );

        let mem_properties = unsafe {
            instance.get_instance()
                .get_physical_device_memory_properties(device)
        };

        VkPhysicalDevice {
            device: device,
            mem_properties: mem_properties
        }
    }

    fn pick_physical_device(
        instance: &ash::Instance,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR
    ) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Physical Devices.")
        };

        let mut result = None;
        for &physical_device in physical_devices.iter() {
            if Self::is_physical_device_suitable(instance, physical_device, surface_loader, surface) {
                if result.is_none() {
                    result = Some(physical_device)
                }
            }
        }

        match result {
            None => panic!("Failed to find a suitable GPU."),
            Some(physical_device) => physical_device,
        }
    }

    fn is_physical_device_suitable(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR
    ) -> bool {
        let _device_features = unsafe { instance.get_physical_device_features(physical_device) };
        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
        if device_properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
            return false;
        }

        let indices = Self::find_queue_family(instance, physical_device, surface_loader, surface);

        let is_queue_family_supported = indices.is_complete();
        let is_device_extension_supported =
            Self::check_device_extension_support(instance, physical_device);
        let is_swapchain_supported = if is_device_extension_supported {
            let swapchain_support = VkSwapchain::query_swapchain_support(physical_device, surface_loader, surface);
            !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
        } else {
            false
        };

        return is_queue_family_supported
            && is_device_extension_supported
            && is_swapchain_supported;
    }

    fn check_device_extension_support(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        let available_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .expect("Failed to get device extension properties.")
        };

        let mut available_extension_names = vec![];
        for extension in available_extensions.iter() {
            let extension_name = utility::tools::vk_to_string(&extension.extension_name);
            available_extension_names.push(extension_name);
        }

        let mut required_extensions = std::collections::HashSet::new();
        for extension in DEVICE_EXTENSIONS.iter() {
            required_extensions.insert(extension.to_string());
        }

        for extension_name in available_extension_names.iter() {
            required_extensions.remove(extension_name);
        }

        return required_extensions.is_empty();
    }

    fn find_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = QueueFamilyIndices {
            graphics_family: None,
            present_family: None
        };

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }

            let is_present_support = unsafe {
                surface_loader
                    .get_physical_device_surface_support(
                        physical_device,
                        index as u32,
                        surface,
                    )
            };
            if queue_family.queue_count > 0 && is_present_support {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }

    pub fn get_device(&self) -> vk::PhysicalDevice {
        self.device
    }

    pub fn get_mem_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.mem_properties
    }
}


impl VkLogicalDevice {
    pub fn new(
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
    ) -> Rc<Self> {
        let indices = VkPhysicalDevice::find_queue_family(instance.get_instance(), physical_device.device, instance.get_surface_loader(), *instance.get_surface());

        let mut unique_queue_families = std::collections::HashSet::new();
        unique_queue_families.insert(indices.graphics_family.unwrap());
        unique_queue_families.insert(indices.present_family.unwrap());

        let queue_priorities = [1.0_f32];
        let mut queue_create_infos = vec![];
        for &_queue_family in unique_queue_families.iter() {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: indices.graphics_family.unwrap(),
                p_queue_priorities: queue_priorities.as_ptr(),
                queue_count: queue_priorities.len() as u32,
            };
            queue_create_infos.push(queue_create_info);
        }

        let physical_device_features = vk::PhysicalDeviceFeatures {
            ..Default::default()
        };

        let enable_extension_names: [*const c_char; 1] = [
            ash::extensions::khr::Swapchain::name().as_ptr(),
        ];

        let requred_validation_layer_raw_names: Vec<CString> = VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let enable_layer_names: Vec<*const c_char> = requred_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_layer_count: if VALIDATION.is_enable {
                enable_layer_names.len()
            } else {
                0
            } as u32,
            pp_enabled_layer_names: if VALIDATION.is_enable {
                enable_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_extension_count: enable_extension_names.len() as u32,
            pp_enabled_extension_names: enable_extension_names.as_ptr(),
            p_enabled_features: &physical_device_features,
        };

        let device: ash::Device = unsafe {
            instance.get_instance()
                .create_device(physical_device.device, &device_create_info, None)
                .expect("Failed to create logical Device!")
        };

        Rc::new(VkLogicalDevice {
            device: device,
            queue_indices: indices
        })
    }

    pub fn get_graphics_queue(&self) -> vk::Queue {
        unsafe { self.device.get_device_queue(self.queue_indices.graphics_family.unwrap(), 0) }
    }

    pub fn get_present_queue(&self) -> vk::Queue {
        unsafe { self.device.get_device_queue(self.queue_indices.present_family.unwrap(), 0) }
    }

    pub fn get_device(&self) -> &ash::Device {
        &self.device
    }

    pub fn get_queue_family_indices(&self) -> &QueueFamilyIndices {
        &self.queue_indices
    }
}

impl Drop for VkLogicalDevice {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}