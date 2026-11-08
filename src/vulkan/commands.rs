use std::error::Error;

use ash::vk;

use crate::vulkan::device::Device;

pub struct Command {
    pool: vk::CommandPool,
}

impl Command {
    pub fn new(device: &Device) -> Result<Self, Box<dyn Error>> {
        log::debug!("Creating Command pool");
        let create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(device.graphics_queue_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let pool = unsafe { device.device.create_command_pool(&create_info, None)? };

        Ok(Self { pool })
    }

    pub fn allocate_buffer(
        &self,
        device: &Device,
        frames_in_flight: u32,
    ) -> Result<Vec<vk::CommandBuffer>, Box<dyn Error>> {
        log::debug!("Allocating {frames_in_flight} command buffers");
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.pool)
            .command_buffer_count(frames_in_flight);

        let buffers = unsafe { device.device.allocate_command_buffers(&allocate_info)? };

        Ok(buffers)
    }

    pub fn destroy(&self, device: &Device) {
        unsafe { device.device.destroy_command_pool(self.pool, None) };
    }
}
