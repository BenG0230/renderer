use std::error::Error;

use ash::vk::{self, DescriptorSet, Sampler};

use crate::vulkan::{
    allocation::{Image, VulkanAllocator},
    descriptor::Descriptor,
    device::Device,
};

pub struct Texture {
    image: Image,
    pub descriptor_set: vk::DescriptorSet,
}

impl Texture {
    pub fn new(
        image: Image,
        device: &Device,
        descriptor: &Descriptor,
        sampler: Sampler,
    ) -> Result<Self, Box<dyn Error>> {
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor.pool)
            .set_layouts(std::slice::from_ref(&descriptor.tex_layout));

        let descriptor_sets = unsafe { device.device.allocate_descriptor_sets(&alloc_info)? };
        let descriptor_set = descriptor_sets[0];

        let image_info = vk::DescriptorImageInfo::default()
            .sampler(sampler)
            .image_view(image.image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        let descriptor_write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&image_info));

        unsafe {
            device
                .device
                .update_descriptor_sets(&[descriptor_write], &[])
        };

        Ok(Self {
            image,
            descriptor_set,
        })
    }

    pub fn destroy(&mut self, device: &Device, allocator: &VulkanAllocator) {
        self.image.destroy(device, allocator);
    }
}
