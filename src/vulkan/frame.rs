use std::error::Error;

use ash::vk;

use crate::vulkan::{
    device::Device,
    sync::{create_fence, create_semaphore},
};

pub struct Frame {
    pub command_buffer: vk::CommandBuffer,
    pub image_available: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}

impl Frame {
    pub fn new(command_buffer: vk::CommandBuffer, device: &Device) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            command_buffer,
            image_available: create_semaphore(device)?,
            in_flight_fence: create_fence(device)?,
        })
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.device.destroy_semaphore(self.image_available, None);
            device.device.destroy_fence(self.in_flight_fence, None);
        };
    }
}
