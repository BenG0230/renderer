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
    allocation: vk_mem::Allocation,
}

impl Buffer {
    pub fn vertex_buffer(
        allocator: &VulkanAllocator,
        data: &[super::renderer::Vertex],
    ) -> Result<Self, Box<dyn Error>> {
        log::debug!("Allocating vertex buffer");
        let buffer_info = vk::BufferCreateInfo::default()
            .size((size_of::<super::renderer::Vertex>() * data.len()) as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::Auto,
            flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE
                | vk_mem::AllocationCreateFlags::MAPPED,
            ..Default::default()
        };

        let (buffer, allocation) = unsafe {
            allocator
                .allocator
                .create_buffer(&buffer_info, &alloc_info)?
        };

        let info = allocator.allocator.get_allocation_info(&allocation);

        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const u8,
                info.mapped_data as *mut u8,
                size_of::<super::renderer::Vertex>() * data.len(),
            );
        }

        Ok(Self { buffer, allocation })
    }

    pub fn index_buffer(allocator: &VulkanAllocator, data: &[u32]) -> Result<Self, Box<dyn Error>> {
        log::debug!("Allocating index buffer");
        let buffer_info = vk::BufferCreateInfo::default()
            .size((size_of::<u32>() * data.len()) as u64)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::Auto,
            flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE
                | vk_mem::AllocationCreateFlags::MAPPED,
            ..Default::default()
        };
        let (buffer, allocation) = unsafe {
            allocator
                .allocator
                .create_buffer(&buffer_info, &alloc_info)?
        };

        let info = allocator.allocator.get_allocation_info(&allocation);

        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const u8,
                info.mapped_data as *mut u8,
                size_of::<u32>() * data.len(),
            );
        }
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
            usage: vk_mem::MemoryUsage::Auto,
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
