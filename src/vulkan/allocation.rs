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
            usage: vk_mem::MemoryUsage::AutoPreferHost,
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
    pub image_view: vk::ImageView,
    allocation: vk_mem::Allocation,
}

impl Image {
    pub fn create_image(
        allocator: &VulkanAllocator,
        device: &Device,
        format: vk::Format,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        log::debug!("allocating image");
        let image_info = &vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::AutoPreferDevice,
            ..Default::default()
        };

        let (image, allocation) =
            unsafe { allocator.allocator.create_image(image_info, &alloc_info)? };

        let view_create_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(image_info.format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1),
            );
        let image_view = unsafe { device.device.create_image_view(&view_create_info, None) }?;

        Ok(Self {
            image,
            image_view,
            allocation,
        })
    }

    pub fn depth_image(
        allocator: &VulkanAllocator,
        device: &Device,
        depth_format: vk::Format,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        log::debug!("allocating depth image");
        let create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(depth_format)
            .extent(vk::Extent3D {
                width: width,
                height: height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::AutoPreferDevice,
            ..Default::default()
        };

        let (image, allocation) = unsafe {
            allocator
                .allocator
                .create_image(&create_info, &alloc_info)?
        };

        let view_create_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(depth_format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            });

        let image_view = unsafe { device.device.create_image_view(&view_create_info, None)? };

        Ok(Self {
            image,
            image_view,
            allocation,
        })
    }

    pub fn destroy(&mut self, device: &Device, allocator: &VulkanAllocator) {
        unsafe {
            allocator
                .allocator
                .destroy_image(self.image, &mut self.allocation);

            device.device.destroy_image_view(self.image_view, None);
        }
    }
}
