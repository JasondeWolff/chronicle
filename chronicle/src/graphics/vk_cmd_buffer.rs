use std::rc::Rc;
use std::ptr;

use ash::version::DeviceV1_0;
use ash::vk;

use crate::graphics::*;

pub struct VkCmdBuffer {
    device: Rc<VkLogicalDevice>,
    cmd_pool: Rc<VkCmdPool>,
    cmd_buffer: vk::CommandBuffer
}

impl VkCmdBuffer {
    pub fn new(device: Rc<VkLogicalDevice>, cmd_pool: Rc<VkCmdPool>, count: u32) -> Vec<Self> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: count,
            command_pool: cmd_pool.get_cmd_pool(),
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffers = unsafe {
            device.get_device()
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers.")
        };

        let mut cmd_buffers = Vec::new();
        for command_buffer in command_buffers {
            cmd_buffers.push(VkCmdBuffer {
                device: device.clone(),
                cmd_pool: cmd_pool.clone(),
                cmd_buffer: command_buffer
            });
        }
        cmd_buffers
    }

    pub fn submit(&self, wait_semaphores: Option<&Vec<&VkSemaphore>>, signal_semaphores: Option<&Vec<&VkSemaphore>>, fence: Option<&VkFence>) {
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let mut wait_semaphores_raw = Vec::new();
        let mut signal_semaphores_raw = Vec::new();

        let (wait_semaphore_count, p_wait_semaphores) = match wait_semaphores {
            Some(wait_semaphores) => {
                for wait_semaphore in wait_semaphores {
                    wait_semaphores_raw.push(*wait_semaphore.get_semaphore());
                }

                (wait_semaphores.len() as u32, wait_semaphores_raw.as_ptr())
            },
            None => (0, ptr::null())
        };

        let (signal_semaphore_count, p_signal_semaphores) = match signal_semaphores {
            Some(signal_semaphores) => {
                for signal_semaphore in signal_semaphores {
                    signal_semaphores_raw.push(*signal_semaphore.get_semaphore());
                }

                (signal_semaphores.len() as u32, signal_semaphores_raw.as_ptr())
            },
            None => (0, ptr::null())
        };
        
        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphore_count,
            p_wait_semaphores: p_wait_semaphores,
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.cmd_buffer,
            signal_semaphore_count: signal_semaphore_count,
            p_signal_semaphores: p_signal_semaphores,
        }];

        let fence = match fence {
            Some(fence) => *fence.get_fence(),
            None => vk::Fence::null()
        };

        unsafe {
            self.device.get_device()
                .queue_submit(
                    self.device.get_graphics_queue(),
                    &submit_infos,
                    fence,
                )
                .expect("Failed to execute queue submit.");
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.device.get_device()
                .reset_command_buffer(
                    self.cmd_buffer,
                    vk::CommandBufferResetFlags::empty()
                )
                .expect("Failed to execute queue reset.");
        }
    }

    pub fn begin(&self, flags: vk::CommandBufferUsageFlags) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: flags,
        };

        unsafe {
            self.device.get_device()
                .begin_command_buffer(self.cmd_buffer, &command_buffer_begin_info)
                .expect("Failed to begin Command Buffer.");
        }
    }

    pub fn end(&self) {
        unsafe {
            self.device.get_device()
                .end_command_buffer(self.cmd_buffer)
                .expect("Failed to end Command Buffer.");
        }
    }

    pub fn set_viewport(&self, extent: &vk::Extent2D) {
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: *extent,
        }];

        unsafe {
            self.device.get_device()
                .cmd_set_viewport(
                    self.cmd_buffer,
                    0,
                    &viewports
                );

            self.device.get_device()
                .cmd_set_scissor(
                    self.cmd_buffer, 
                    0, 
                    &scissors
                );
        }
    }

    pub fn begin_render_pass(&self, render_pass: &VkRenderPass, swapchain: &VkSwapchain, frame_idx: usize) {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: *render_pass.get_render_pass(),
            framebuffer: *swapchain.get_framebuffer(frame_idx),
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: *swapchain.get_extent(),
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        unsafe {
            self.device.get_device().cmd_begin_render_pass(
                self.cmd_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    pub fn end_render_pass(&self) {
        unsafe {
            self.device.get_device()
                .cmd_end_render_pass(self.cmd_buffer);
        }
    }

    pub fn bind_graphics_pipeline(&self, pipeline: &VkPipeline) {
        unsafe {
            self.device.get_device()
                .cmd_bind_pipeline(
                    self.cmd_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    *pipeline.get_pipeline(),
                );
        }
    }

    pub fn bind_vertex_buffer(&self, vertex_buffer: &VkVertexBuffer) {
        unsafe {
            self.device.get_device()
                .cmd_bind_vertex_buffers(
                    self.cmd_buffer,
                    0,
                    &[vertex_buffer.get_buffer()],
                    &[0_u64]
                );
        }
    }

    pub fn bind_index_buffer(&self, index_buffer: &VkIndexBuffer) {
        unsafe {
            self.device.get_device()
                .cmd_bind_index_buffer(
                    self.cmd_buffer,
                    index_buffer.get_buffer(),
                    0,
                    vk::IndexType::UINT32
                );
        }
    }

    pub fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        unsafe {
            self.device.get_device()
                .cmd_draw(
                    self.cmd_buffer,
                    vertex_count,
                    instance_count,
                    first_vertex,
                    first_instance
                );
        }
    }

    pub fn draw_indexed(&self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: u32, first_instance: u32) {
        unsafe {
            self.device.get_device()
                .cmd_draw_indexed(
                    self.cmd_buffer,
                    index_count,
                    instance_count,
                    first_index,
                    vertex_offset as i32,
                    first_instance
                );
        }
    }

    pub fn copy_buffers(&self, src_buffer: &VkBuffer, dst_buffer: &VkBuffer) {
        assert_eq!(src_buffer.get_size(), dst_buffer.get_size(), "Failed to copy buffers.");
        
        let copy_regions = [vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: src_buffer.get_size(),
        }];

        unsafe {
            self.device.get_device()
                .cmd_copy_buffer(
                    self.cmd_buffer,
                    src_buffer.get_buffer(),
                    dst_buffer.get_buffer(),
                    &copy_regions
                );
        }
    }
}

impl Drop for VkCmdBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.get_device()
                .free_command_buffers(
                    self.cmd_pool.get_cmd_pool(),
                    &[self.cmd_buffer]
                );
        }
    }
}