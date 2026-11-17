use std::error::Error;

use ash::{Instance, vk};
use vk_mem::{Alloc, Allocator, AllocatorCreateInfo};

use crate::vulkan::device::Device;

pub struct VulkanAllocator {
    pub allocator: Allocator,
}

impl VulkanAllocator {
    pub fn new(instance: &Instance, device: &Device) -> Result<Self, Box<dyn Error>> {
        let create_info =
            AllocatorCreateInfo::new(instance, &device.device, device.physical_device);

        let allocator = unsafe { Allocator::new(create_info)? };

        Ok(Self { allocator })
    }
}

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub allocation: vk_mem::Allocation,
}

impl Buffer {
    pub fn vertex_buffer(allocator: &VulkanAllocator, size: u64) -> Result<Self, Box<dyn Error>> {
        log::debug!("Allocating vertex buffer");
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::AutoPreferDevice,
            ..Default::default()
        };

        let (buffer, allocation) = unsafe {
            allocator
                .allocator
                .create_buffer(&buffer_info, &alloc_info)?
        };

        Ok(Self { buffer, allocation })
    }

    pub fn index_buffer(allocator: &VulkanAllocator, size: u64) -> Result<Self, Box<dyn Error>> {
        log::debug!("Allocating index buffer");
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::AutoPreferDevice,
            ..Default::default()
        };
        let (buffer, allocation) = unsafe {
            allocator
                .allocator
                .create_buffer(&buffer_info, &alloc_info)?
        };

        Ok(Self { buffer, allocation })
    }

    pub fn staging_buffer(allocator: &VulkanAllocator, size: u64) -> Result<Self, Box<dyn Error>> {
        log::debug!("Allocating staging buffer");
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::Auto,
            flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
            ..Default::default()
        };

        let (buffer, allocation) = unsafe {
            allocator
                .allocator
                .create_buffer(&buffer_info, &alloc_info)?
        };

        Ok(Self { buffer, allocation })
    }

    pub fn destroy(&mut self, allocator: &VulkanAllocator) {
        unsafe {
            allocator
                .allocator
                .destroy_buffer(self.buffer, &mut self.allocation)
        };
    }
}

pub struct Image {
    pub image: vk::Image,
    allocation: vk_mem::Allocation,
}

impl Image {
    pub fn create_image(
        allocator: &VulkanAllocator,
        image_info: &vk::ImageCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        log::debug!("allocating image");
        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::AutoPreferDevice,
            flags: vk_mem::AllocationCreateFlags::DEDICATED_MEMORY,
            ..Default::default()
        };

        let (image, allocation) =
            unsafe { allocator.allocator.create_image(image_info, &alloc_info)? };

        Ok(Self { image, allocation })
    }

    pub fn destroy(&mut self, allocator: &VulkanAllocator) {
        unsafe {
            allocator
                .allocator
                .destroy_image(self.image, &mut self.allocation);
        }
    }
}
