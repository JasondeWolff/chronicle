use std::rc::Rc;
use std::collections::VecDeque;

use ash::vk;

use crate::graphics::*;

struct InFlightCmdBuffer {
    fence: Rc<VkFence>,
    cmd_buffer: RcCell<VkCmdBuffer>
}

pub struct VkCmdQueue {
    device: Rc<VkLogicalDevice>,
    desc_pool: Rc<VkDescriptorPool>,
    queue: vk::Queue,
    cmd_pool: Rc<VkCmdPool>,
    _queue_type: VkQueueType,

    busy_cmd_buffers: VecDeque<InFlightCmdBuffer>,
    idle_cmd_buffers: VecDeque<RcCell<VkCmdBuffer>>
}

pub enum VkQueueType {
    GRAPHICS,
    PRESENT
}

impl VkCmdQueue {
    pub fn new(
        device: Rc<VkLogicalDevice>,
        desc_pool: Rc<VkDescriptorPool>,
        queue: vk::Queue,
        queue_type: VkQueueType
    ) -> Self {
        let cmd_pool = VkCmdPool::new(device.clone());

        VkCmdQueue {
            device: device,
            desc_pool: desc_pool,
            queue: queue,
            cmd_pool: cmd_pool,
            _queue_type: queue_type,
            busy_cmd_buffers: VecDeque::new(),
            idle_cmd_buffers: VecDeque::new()
        }
    }

    pub fn get_cmd_queue(&self) -> vk::Queue {
        self.queue
    }

    pub fn get_cmd_buffer(&mut self) -> RcCell<VkCmdBuffer> {
        match self.idle_cmd_buffers.pop_front() {
            Some(idle_cmd_buffer) => idle_cmd_buffer,
            None => {
                RcCell::new(VkCmdBuffer::new(
                    self.device.clone(),
                    self.cmd_pool.clone(),
                    self.desc_pool.clone()
                ))
            }
        }
    }

    pub fn submit_cmd_buffer(&mut self,
        cmd_buffer: RcCell<VkCmdBuffer>,
        wait_semaphores: Option<&Vec<&VkSemaphore>>,
        signal_semaphores: Option<&Vec<&VkSemaphore>>
    ) -> Rc<VkFence> {
        self.submit_cmd_buffers(
            &vec![cmd_buffer],
            wait_semaphores,
            signal_semaphores
        )
    }

    pub fn submit_cmd_buffers(&mut self,
        cmd_buffers: &Vec<RcCell<VkCmdBuffer>>,
        wait_semaphores: Option<&Vec<&VkSemaphore>>,
        signal_semaphores: Option<&Vec<&VkSemaphore>>
    ) -> Rc<VkFence> {
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let mut wait_semaphores_raw = Vec::new();
        let mut signal_semaphores_raw = Vec::new();
        let mut cmd_buffers_raw = Vec::new();

        for cmd_buffer in cmd_buffers {
            cmd_buffers_raw.push(cmd_buffer.as_ref().get_cmd_buffer());
        }

        let (wait_semaphore_count, p_wait_semaphores) = match wait_semaphores {
            Some(wait_semaphores) => {
                for wait_semaphore in wait_semaphores {
                    wait_semaphores_raw.push(*wait_semaphore.get_semaphore());
                }

                (wait_semaphores.len() as u32, wait_semaphores_raw.as_ptr())
            },
            None => (0, std::ptr::null())
        };

        let (signal_semaphore_count, p_signal_semaphores) = match signal_semaphores {
            Some(signal_semaphores) => {
                for signal_semaphore in signal_semaphores {
                    signal_semaphores_raw.push(*signal_semaphore.get_semaphore());
                }

                (signal_semaphores.len() as u32, signal_semaphores_raw.as_ptr())
            },
            None => (0, std::ptr::null())
        };

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: std::ptr::null(),
            wait_semaphore_count: wait_semaphore_count,
            p_wait_semaphores: p_wait_semaphores,
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: cmd_buffers_raw.len() as u32,
            p_command_buffers: cmd_buffers_raw.as_ptr(),
            signal_semaphore_count: signal_semaphore_count,
            p_signal_semaphores: p_signal_semaphores,
        }];

        let fence = VkFence::new(self.device.clone(), false);
    
        unsafe {
            self.device.get_device()
                .queue_submit(
                    self.queue,
                    &submit_infos,
                    fence.get_fence(),
                )
                .expect("Failed to execute queue submit.");
        }

        for cmd_buffer in cmd_buffers {
            self.busy_cmd_buffers.push_back(InFlightCmdBuffer {
                fence: fence.clone(),
                cmd_buffer: cmd_buffer.clone()
            });
        }

        fence
    }

    pub fn process_busy_cmds(&mut self) {
        while !self.busy_cmd_buffers.is_empty() {
            if let Some(inflight_cmd_buffer) = self.busy_cmd_buffers.front() {
                if inflight_cmd_buffer.fence.is_completed() {
                    let inflight_cmd_buffer = self.busy_cmd_buffers.pop_front().unwrap();
                    inflight_cmd_buffer.cmd_buffer.as_mut().reset();
                    self.idle_cmd_buffers.push_back(inflight_cmd_buffer.cmd_buffer);
                }
            } else {
                break;
            }
        }
    }
}