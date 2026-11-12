use std::error::Error;

use ash::vk;

use crate::vulkan::device::Device;

pub fn create_semaphore(device: &Device) -> Result<vk::Semaphore, Box<dyn Error>> {
    let create_info = vk::SemaphoreCreateInfo::default();

    Ok(unsafe { device.device.create_semaphore(&create_info, None)? })
}

pub fn create_fence(device: &Device) -> Result<vk::Fence, Box<dyn Error>> {
    let create_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

    Ok(unsafe { device.device.create_fence(&create_info, None)? })
}
