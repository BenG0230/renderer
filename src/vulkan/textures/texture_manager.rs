use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
};

use ash::vk;
use image::ImageReader;

use crate::vulkan::{
    allocation::{Buffer, Image, VulkanAllocator},
    device::Device,
    sync::create_fence,
    textures::texture::Texture,
};

const STAGING_BUFFER_SIZE: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy)]
pub struct TextureHandle(usize);

pub struct TextureManager {
    textures: Vec<Texture>,
    loaded_textures: HashMap<PathBuf, TextureHandle>,

    command_buffer: vk::CommandBuffer,
    upload_fence: vk::Fence,

    staging_buffer: Buffer,
}

impl TextureManager {
    pub fn new(
        allocator: &VulkanAllocator,
        device: &Device,
        command_buffer: vk::CommandBuffer,
    ) -> Result<Self, Box<dyn Error>> {
        let staging_buffer = Buffer::staging_buffer(allocator, STAGING_BUFFER_SIZE)?;

        let upload_fence = create_fence(device)?;

        Ok(Self {
            textures: Vec::new(),
            loaded_textures: HashMap::new(),

            command_buffer,
            upload_fence,

            staging_buffer,
        })
    }

    pub fn load_texture(
        &mut self,
        path: &Path,
        allocator: &VulkanAllocator,
        device: &Device,
    ) -> Result<TextureHandle, Box<dyn Error>> {
        let path = &path.canonicalize()?;

        if let Some(handle) = self.loaded_textures.get(path) {
            return Ok(*handle);
        }

        let image = ImageReader::open(path)?.decode()?.to_rgba8();

        let (width, height) = image.dimensions();
        let data = image.into_raw();

        log::debug!(
            "Image size: {:.2}MiB",
            data.len() as f64 / (1024.0 * 1024.0)
        );

        let texture = self.load_texture_data(&data, width, height, allocator, device)?;

        let handle = TextureHandle(self.textures.len());
        self.textures.push(texture);
        self.loaded_textures.insert(path.to_path_buf(), handle);

        Ok(handle)
    }

    pub fn load_texture_data(
        &mut self,
        data: &[u8],
        width: u32,
        height: u32,
        allocator: &VulkanAllocator,
        device: &Device,
    ) -> Result<Texture, Box<dyn Error>> {
        self.copy_to_staging(allocator, data)?;

        let image = Image::create_image(allocator, device, vk::Format::R8G8B8_SRGB, width, height)?;

        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            device
                .device
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())?;
            device
                .device
                .begin_command_buffer(self.command_buffer, &begin_info)?;
        };

        let barrier = vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::NONE)
            .src_access_mask(vk::AccessFlags2::NONE)
            .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(image.image)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1)
                    .base_array_layer(0)
                    .base_mip_level(1),
            );

        let depedency_info =
            vk::DependencyInfo::default().image_memory_barriers(std::slice::from_ref(&barrier));

        unsafe {
            device
                .device
                .cmd_pipeline_barrier2(self.command_buffer, &depedency_info);
        };

        let copy = vk::BufferImageCopy::default()
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .mip_level(0)
                    .base_array_layer(0),
            );

        unsafe {
            device.device.cmd_copy_buffer_to_image(
                self.command_buffer,
                self.staging_buffer.buffer,
                image.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[copy],
            )
        };

        let barrier = vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
            .dst_access_mask(vk::AccessFlags2::SHADER_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image(image.image)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1)
                    .base_array_layer(0)
                    .base_mip_level(1),
            );

        let depedency_info =
            vk::DependencyInfo::default().image_memory_barriers(std::slice::from_ref(&barrier));

        unsafe {
            device
                .device
                .cmd_pipeline_barrier2(self.command_buffer, &depedency_info);

            device.device.end_command_buffer(self.command_buffer)?;

            let submit_info = vk::SubmitInfo::default()
                .command_buffers(std::slice::from_ref(&self.command_buffer));

            device.device.reset_fences(&[self.upload_fence])?;

            device
                .device
                .queue_submit(device.graphics_queue, &[submit_info], self.upload_fence)?;

            device
                .device
                .wait_for_fences(&[self.upload_fence], true, u64::MAX)?;
        };

        Ok(Texture::new(image, device)?)
    }

    fn copy_to_staging(
        &mut self,
        allocator: &VulkanAllocator,
        data: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        unsafe {
            let mapped = allocator
                .allocator
                .map_memory(&mut self.staging_buffer.allocation)?;

            std::ptr::copy_nonoverlapping(data.as_ptr() as *const u8, mapped, data.len());

            allocator
                .allocator
                .unmap_memory(&mut self.staging_buffer.allocation);
        };

        Ok(())
    }

    pub fn destroy(&mut self, device: &Device, allocator: &VulkanAllocator) {
        for texture in &mut self.textures {
            texture.destroy(device, allocator);
        }

        self.staging_buffer.destroy(allocator);

        unsafe { device.device.destroy_fence(self.upload_fence, None) };
    }
}
