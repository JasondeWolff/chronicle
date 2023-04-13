use std::rc::Rc;

use ash::vk;

use crate::app;
use crate::graphics::*;

const INDICES_DATA: [u32; 6] = [0, 1, 2, 2, 3, 0];

pub struct VkIndexBuffer {
    index_buffer: VkBuffer,
    upload_cmd_buffer: Vec<VkCmdBuffer>,
    index_count: u32
}

impl VkIndexBuffer {
    pub fn new(
        device: Rc<VkLogicalDevice>,
        physical_device: &VkPhysicalDevice,
        cmd_pool: Rc<VkCmdPool>,
        indices: &Vec<u32>
    ) -> Self {
        //let indices = vec![INDICES_DATA[0].clone(), INDICES_DATA[1].clone(), INDICES_DATA[2].clone(), INDICES_DATA[3].clone(), INDICES_DATA[4].clone(), INDICES_DATA[5].clone()];

        let size = (std::mem::size_of::<u32>() * indices.len()) as u64;
        
        let staging_buffer = VkBuffer::new(
            device.clone(),
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            physical_device.get_mem_properties()
        );

        unsafe {
            let data_ptr = staging_buffer.map() as *mut u32;
            data_ptr.copy_from_nonoverlapping(indices.as_ptr(), indices.len());
            staging_buffer.unmap();
        }

        let index_buffer = VkBuffer::new(
            device.clone(),
            size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            physical_device.get_mem_properties()
        );

        let cmd_buffers = VkCmdBuffer::new(device, cmd_pool, 1);
        let cmd_buffer = &cmd_buffers[0];
        cmd_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        cmd_buffer.copy_buffers(&staging_buffer, &index_buffer);
        cmd_buffer.end();
        cmd_buffer.submit(None, None, None);
        app().graphics().wait_idle();

        VkIndexBuffer {
            index_buffer: index_buffer,
            upload_cmd_buffer: cmd_buffers,
            index_count: indices.len() as u32
        }
    }

    pub fn get_buffer(&self) -> vk::Buffer {
        self.index_buffer.get_buffer()
    }

    pub fn index_count(&self) -> u32 {
        self.index_count
    }
}