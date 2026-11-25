use std::error::Error;

use ash::vk;

use crate::vulkan::device::Device;

pub struct Descriptor {
    pub pool: vk::DescriptorPool,
    pub tex_layout: vk::DescriptorSetLayout,
}

impl Descriptor {
    pub fn new(device: &Device) -> Result<Self, Box<dyn Error>> {
        let tex_binding = vk::DescriptorSetLayoutBinding::default()
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let tex_layout_ci = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(std::slice::from_ref(&tex_binding));

        let tex_layout = unsafe {
            device
                .device
                .create_descriptor_set_layout(&tex_layout_ci, None)?
        };

        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(100);

        let create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(std::slice::from_ref(&pool_size))
            .max_sets(100);

        let pool = unsafe { device.device.create_descriptor_pool(&create_info, None) }?;

        Ok(Self { pool, tex_layout })
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.device.destroy_descriptor_pool(self.pool, None);
            device
                .device
                .destroy_descriptor_set_layout(self.tex_layout, None);
        };
    }
}
