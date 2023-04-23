use ash::vk;

use crate::graphics::*;
use utility::constants::{VALIDATION, ENABLE_EXTENSION_NAMES};

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct VkLogicalDevice {
    device: ash::Device,
    queue_indices: QueueFamilyIndices,

    raytracing_loader: ash::extensions::khr::RayTracingPipeline,
    accel_loader: ash::extensions::khr::AccelerationStructure,
    buffer_device_address_loader: ash::extensions::khr::BufferDeviceAddress
}

impl VkLogicalDevice {
    pub fn new(
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
    ) -> Arc<Self> {
        let indices = VkPhysicalDevice::find_queue_family(instance.get_instance(), physical_device.get_device(), instance.get_surface_loader(), *instance.get_surface());

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
                .create_device(physical_device.get_device(), &device_create_info, None)
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