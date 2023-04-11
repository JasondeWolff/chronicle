use std::rc::Rc;
use std::ptr;

use ash::version::DeviceV1_0;
use ash::vk;

use crate::graphics::*;
use utility::constants::MAX_FRAMES_IN_FLIGHT;

pub struct VkCmdBuffer {
    device: Rc<VkLogicalDevice>,
    cmd_buffer: vk::CommandBuffer
}

impl VkCmdBuffer {
    pub fn new(device: Rc<VkLogicalDevice>, cmd_pool: &VkCmdPool, swapchain: &VkSwapchain) -> Vec<Self> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: swapchain.get_framebuffer_count() as u32,
            command_pool: *cmd_pool.get_cmd_pool(),
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
                cmd_buffer: command_buffer
            });
        }
        cmd_buffers
    }

    pub fn get_cmd_buffer(&self)-> &vk::CommandBuffer {
        &self.cmd_buffer
    }

    pub fn submit(&self, wait_semaphores: &Vec<&VkSemaphore>, signal_semaphores: &Vec<&VkSemaphore>, fence: &VkFence) {
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let mut wait_semaphores_raw = Vec::new();
        for wait_semaphore in wait_semaphores {
            wait_semaphores_raw.push(*wait_semaphore.get_semaphore());
        }

        let mut signal_semaphores_raw = Vec::new();
        for signal_semaphore in signal_semaphores {
            signal_semaphores_raw.push(*signal_semaphore.get_semaphore());
        }

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores_raw.len() as u32,
            p_wait_semaphores: wait_semaphores_raw.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.cmd_buffer,
            signal_semaphore_count: signal_semaphores_raw.len() as u32,
            p_signal_semaphores: signal_semaphores_raw.as_ptr(),
        }];

        unsafe {
            self.device.get_device()
                .queue_submit(
                    self.device.get_graphics_queue(),
                    &submit_infos,
                    *fence.get_fence(),
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

    pub fn begin(&self) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
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
            self.device.get_device().cmd_bind_pipeline(
                self.cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                *pipeline.get_pipeline(),
            );
        }
    }

    pub fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        unsafe {
            self.device.get_device().cmd_draw(
                self.cmd_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance
            );
        }
    }
}