use ash::vk;

use crate::graphics::*;
use utility::constants::VALIDATION;

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

// Depenedents chain down E.G.
// "VK_KHR_A"
// "VK_KHR_B" (required for VK_KHR_A)
// "VK_KHR_C" (required for VK_KHR_B)
const DEVICE_EXTENSIONS: [&'static str; 9] = [
    "VK_KHR_swapchain",

    "VK_KHR_device_group",
    "VK_KHR_buffer_device_address",

    "VK_KHR_acceleration_structure",
    "VK_EXT_descriptor_indexing",

    "VK_KHR_ray_tracing_pipeline",

    "VK_KHR_deferred_host_operations",
    "VK_KHR_spirv_1_4",
    "VK_KHR_shader_float_controls"
];

const ENABLE_EXTENSION_NAMES: [*const c_char; 9] = [
    ash::extensions::khr::Swapchain::name().as_ptr(),
    ash::extensions::khr::DeviceGroup::name().as_ptr(),
    ash::extensions::khr::BufferDeviceAddress::name().as_ptr(),
    ash::extensions::khr::AccelerationStructure::name().as_ptr(),
    ash::vk::ExtDescriptorIndexingFn::name().as_ptr(),
    ash::extensions::khr::RayTracingPipeline::name().as_ptr(),
    ash::extensions::khr::DeferredHostOperations::name().as_ptr(),
    ash::vk::KhrSpirv14Fn::name().as_ptr(),
    ash::vk::KhrShaderFloatControlsFn::name().as_ptr()
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
    mem_properties: vk::PhysicalDeviceMemoryProperties,
    max_sample_count: vk::SampleCountFlags,

    raytracing_pipeline_props: vk::PhysicalDeviceRayTracingPipelinePropertiesKHR,
    accel_props: vk::PhysicalDeviceAccelerationStructurePropertiesKHR
}

pub struct VkLogicalDevice {
    device: ash::Device,
    queue_indices: QueueFamilyIndices,

    raytracing_loader: ash::extensions::khr::RayTracingPipeline,
    accel_loader: ash::extensions::khr::AccelerationStructure,
    buffer_device_address_loader: ash::extensions::khr::BufferDeviceAddress
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

        let max_sample_count = Self::max_sample_count(
            instance.get_instance(),
            device
        );

        let (raytracing_pipeline_props, accel_props) = Self::raytracing_properties(
            instance.get_instance(),
            device
        );

        VkPhysicalDevice {
            device,
            mem_properties,
            max_sample_count,
            raytracing_pipeline_props,
            accel_props
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
        let device_features = unsafe { instance.get_physical_device_features(physical_device) };
        
        let mut device_properties = vk::PhysicalDeviceProperties2 {
            s_type: vk::StructureType::PHYSICAL_DEVICE_PROPERTIES_2,
            ..Default::default()
        };
        unsafe { instance.get_physical_device_properties2(physical_device, &mut device_properties) };
        if device_properties.properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
            return false;
        }

        let indices = Self::find_queue_family(instance, physical_device, surface_loader, surface);

        let is_queue_family_supported = indices.is_complete();
        let is_device_extension_supported = Self::check_device_extension_support(instance, physical_device);
        let is_swapchain_supported = if is_device_extension_supported {
            let swapchain_support = VkSwapchain::query_swapchain_support(physical_device, surface_loader, surface);
            !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
        } else {
            false
        };

        return is_queue_family_supported
            && is_device_extension_supported
            && is_swapchain_supported
            && device_features.sampler_anisotropy > 0;
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
                    .expect("Failed to get surface support.")
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

    fn max_sample_count(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice
    ) -> vk::SampleCountFlags {
        let physical_device_properties = unsafe {
            instance.get_physical_device_properties(physical_device)
        };

        let count = std::cmp::min(
            physical_device_properties
                .limits
                .framebuffer_color_sample_counts,
            physical_device_properties
                .limits
                .framebuffer_depth_sample_counts,
        );

        if count.contains(vk::SampleCountFlags::TYPE_64) {
            return vk::SampleCountFlags::TYPE_64;
        }
        if count.contains(vk::SampleCountFlags::TYPE_32) {
            return vk::SampleCountFlags::TYPE_32;
        }
        if count.contains(vk::SampleCountFlags::TYPE_16) {
            return vk::SampleCountFlags::TYPE_16;
        }
        if count.contains(vk::SampleCountFlags::TYPE_8) {
            return vk::SampleCountFlags::TYPE_8;
        }
        if count.contains(vk::SampleCountFlags::TYPE_4) {
            return vk::SampleCountFlags::TYPE_4;
        }
        if count.contains(vk::SampleCountFlags::TYPE_2) {
            return vk::SampleCountFlags::TYPE_2;
        }

        vk::SampleCountFlags::TYPE_1
    }

    fn raytracing_properties(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice
    ) -> (vk::PhysicalDeviceRayTracingPipelinePropertiesKHR, vk::PhysicalDeviceAccelerationStructurePropertiesKHR) {
        let mut rt_properties = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR {
            s_type: vk::StructureType::PHYSICAL_DEVICE_RAY_TRACING_PIPELINE_PROPERTIES_KHR,
            ..Default::default()
        };
        let mut accel_properties = vk::PhysicalDeviceAccelerationStructurePropertiesKHR  {
            p_next: &mut rt_properties as *mut vk::PhysicalDeviceRayTracingPipelinePropertiesKHR as *mut std::ffi::c_void,
            ..Default::default()
        };
        let mut device_properties = vk::PhysicalDeviceProperties2 {
            p_next: &mut accel_properties as *mut vk::PhysicalDeviceAccelerationStructurePropertiesKHR as *mut std::ffi::c_void,
            s_type: vk::StructureType::PHYSICAL_DEVICE_PROPERTIES_2,
            
            ..Default::default()
        };

        unsafe {
            instance.get_physical_device_properties2(physical_device, &mut device_properties);
        }

        (rt_properties, accel_properties)
    }

    pub fn get_device(&self) -> vk::PhysicalDevice {
        self.device
    }

    pub fn get_mem_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.mem_properties
    }

    pub fn get_max_sample_count(&self) -> vk::SampleCountFlags {
        self.max_sample_count
    }

    pub fn get_raytracing_properties(&self) -> &vk::PhysicalDeviceRayTracingPipelinePropertiesKHR {
        &self.raytracing_pipeline_props
    }

    pub fn get_accel_properties(&self) -> &vk::PhysicalDeviceAccelerationStructurePropertiesKHR {
        &self.accel_props
    }
}


impl VkLogicalDevice {
    pub fn new(
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
    ) -> Arc<Self> {
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
            sampler_anisotropy: vk::TRUE,
            ..Default::default()
        };

        let mut buffer_device_address_features = vk::PhysicalDeviceBufferDeviceAddressFeaturesEXT {
            buffer_device_address: 1,
            ..Default::default()
        };

        let mut acceleration_features = vk::PhysicalDeviceAccelerationStructureFeaturesKHR {
            acceleration_structure: 1,
            p_next: &mut buffer_device_address_features as *mut vk::PhysicalDeviceBufferDeviceAddressFeaturesEXT as *mut std::ffi::c_void,
            ..Default::default()
        };

        let raytracing_features = vk::PhysicalDeviceRayTracingPipelineFeaturesKHR {
            ray_tracing_pipeline: 1,
            p_next: &mut acceleration_features as *mut vk::PhysicalDeviceAccelerationStructureFeaturesKHR as *mut std::ffi::c_void,
            ..Default::default()
        };

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
            p_next: &raytracing_features as *const vk::PhysicalDeviceRayTracingPipelineFeaturesKHR as *const std::ffi::c_void,
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
            enabled_extension_count: ENABLE_EXTENSION_NAMES.len() as u32,
            pp_enabled_extension_names: ENABLE_EXTENSION_NAMES.as_ptr(),
            p_enabled_features: &physical_device_features,
        };

        let device: ash::Device = unsafe {
            instance.get_instance()
                .create_device(physical_device.device, &device_create_info, None)
                .expect("Failed to create logical Device!")
        };

        let raytracing_loader = ash::extensions::khr::RayTracingPipeline::new(instance.get_instance(), &device);
        let accel_loader = ash::extensions::khr::AccelerationStructure::new(instance.get_instance(), &device);
        let buffer_device_address_loader = ash::extensions::khr::BufferDeviceAddress::new(instance.get_instance(), &device);

        Arc::new(VkLogicalDevice {
            device: device,
            queue_indices: indices,
            raytracing_loader: raytracing_loader,
            accel_loader: accel_loader,
            buffer_device_address_loader: buffer_device_address_loader
        })
    }

    pub fn get_graphics_queue(&self) -> vk::Queue {
        unsafe { self.device.get_device_queue(self.queue_indices.graphics_family.unwrap(), 0) }
    }

    pub fn get_present_queue(&self) -> vk::Queue {
        unsafe { self.device.get_device_queue(self.queue_indices.present_family.unwrap(), 0) }
    }

    pub fn wait_idle(&self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait device idle.")
        };
    }

    pub fn get_device(&self) -> &ash::Device {
        &self.device
    }

    pub fn get_queue_family_indices(&self) -> &QueueFamilyIndices {
        &self.queue_indices
    }

    pub fn raytracing_loader(&self) -> &ash::extensions::khr::RayTracingPipeline {
        &self.raytracing_loader
    }

    pub fn accel_loader(&self) -> &ash::extensions::khr::AccelerationStructure {
        &self.accel_loader
    }

    pub fn buffer_device_address_loader(&self) -> &ash::extensions::khr::BufferDeviceAddress {
        &self.buffer_device_address_loader
    }
}

impl Drop for VkLogicalDevice {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}