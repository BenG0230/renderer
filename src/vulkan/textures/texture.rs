use std::error::Error;

use ash::vk;

use crate::vulkan::{
    allocation::{Image, VulkanAllocator},
    device::Device,
};

pub struct Texture {
    image: Image,
    sampler: vk::Sampler,
}

impl Texture {
    pub fn new(image: Image, device: &Device) -> Result<Self, Box<dyn Error>> {
        let create_info = &vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::NEAREST)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .anisotropy_enable(true)
            .max_anisotropy(8.0)
            .max_lod(0.0);

        let sampler = unsafe { device.device.create_sampler(create_info, None)? };

        Ok(Self { image, sampler })
    }

    pub fn destroy(&mut self, device: &Device, allocator: &VulkanAllocator) {
        self.image.destroy(device, allocator);
        unsafe { device.device.destroy_sampler(self.sampler, None) };
    }
}
