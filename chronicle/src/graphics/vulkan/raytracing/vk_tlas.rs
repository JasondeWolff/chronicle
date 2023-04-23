use ash::vk;

use crate::graphics::*;

pub struct VkTlas {
    accel: Option<ArcMutex<VkAccel>>,
}

impl VkTlas {
    pub fn new() -> ArcMutex<Self> {
        let tlas = ArcMutex::new(VkTlas {
            accel: None
        });

        tlas
    }

    pub fn rebuild(&mut self,
        app: &mut VkApp,
        instances: &Vec<VkBlasInstance>,
        build_flags: vk::BuildAccelerationStructureFlagsKHR
    ) {
        let build_flags = build_flags | vk::BuildAccelerationStructureFlagsKHR::ALLOW_UPDATE;
        let instances_size = std::mem::size_of::<VkBlasInstance>() * instances.len();

        let staging_buffer = VkBuffer::new(
            "Tlas staging buffer",
            app.get_device().clone(),
            app.get_allocator(),
            instances_size as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            None
        );

        unsafe {
            let data_ptr = staging_buffer.map() as *mut VkBlasInstance;
            data_ptr.copy_from_nonoverlapping(instances.as_ptr(), instances.len());
            staging_buffer.unmap();
        }

        let instances_buffer = VkBuffer::new(
            "Tlas instances buffer",
            app.get_device(),
            app.get_allocator(),
            instances_size as u64,
            vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            None
        );
        let buffer_address = instances_buffer.get_device_address();

        {
            let cmd_queue = app.get_cmd_queue();
            let mut cmd_queue = cmd_queue.as_mut();
            let cmd_buffer = cmd_queue.get_cmd_buffer(); {
                let cmd_buffer_ref = cmd_buffer.as_ref();
                cmd_buffer_ref.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
                cmd_buffer_ref.copy_buffers(&staging_buffer, &instances_buffer);
                cmd_buffer_ref.end();
            }
            cmd_queue.submit_cmd_buffer(cmd_buffer, None, None);
            app.get_device().wait_idle();
        }

        let cmd_queue = app.get_cmd_queue();
        let mut cmd_queue = cmd_queue.as_mut();
        let cmd_buffer = cmd_queue.get_cmd_buffer(); {
            let mut cmd_buffer_ref = cmd_buffer.as_mut();
            cmd_buffer_ref.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            if !self.accel.is_some() {
                self.accel = Some(cmd_buffer_ref.create_tlas(
                    None,
                    instances.len() as u32,
                    buffer_address,
                    build_flags,
                    false,
                    app.get_physical_device().get_accel_properties()
                ).unwrap());
            } else {
                cmd_buffer_ref.create_tlas(
                    Some(self.accel.as_ref().unwrap().clone()),
                    instances.len() as u32,
                    buffer_address,
                    build_flags,
                    true,
                    app.get_physical_device().get_accel_properties()
                );
            }
            
            cmd_buffer_ref.end();
        }
        cmd_queue.submit_cmd_buffer(cmd_buffer, None, None);
        app.get_device().wait_idle();
    }
}