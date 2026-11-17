use std::error::Error;

use ash::vk;

use crate::vulkan::{
    allocation::{Buffer, VulkanAllocator},
    device::Device,
    models::mesh::Mesh,
    renderer::Vertex,
    sync::create_fence,
};

pub struct MeshHandle(usize);

pub struct MeshManager {
    meshes: Vec<Mesh>,

    command_buffer: vk::CommandBuffer,
    upload_fence: vk::Fence,

    vertex_buffer: Buffer,
    vertex_offset: u64,

    index_buffer: Buffer,
    index_offset: u64,

    staging_buffer: Buffer,
}

impl MeshManager {
    pub fn new(
        allocator: &VulkanAllocator,
        device: &Device,
        command_buffer: vk::CommandBuffer,
    ) -> Result<Self, Box<dyn Error>> {
        let vertex_buffer = Buffer::vertex_buffer(allocator, 32 * 1024 * 1024)?;
        let index_buffer = Buffer::index_buffer(allocator, 32 * 1024 * 1024)?;

        let staging_buffer = Buffer::staging_buffer(allocator, 16 * 1024 * 1024)?;

        Ok(Self {
            meshes: Vec::new(),

            command_buffer,
            upload_fence: create_fence(device)?,

            vertex_buffer,
            vertex_offset: 0,

            index_buffer,
            index_offset: 0,

            staging_buffer,
        })
    }

    pub fn load_mesh(
        &mut self,
        allocator: &VulkanAllocator,
        device: &Device,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<MeshHandle, Box<dyn Error>> {
        let vertex_size = size_of::<Vertex>() * vertices.len();
        let index_size = size_of::<u32>() * indices.len();

        self.copy_to_staging(allocator, vertices, 0)?;
        self.copy_to_staging(allocator, indices, vertex_size)?;

        let vertex_copy = vk::BufferCopy::default()
            .src_offset(0)
            .dst_offset(self.vertex_offset)
            .size(vertex_size as u64);

        let index_copy = vk::BufferCopy::default()
            .src_offset(vertex_size as u64)
            .dst_offset(self.index_offset)
            .size(index_size as u64);

        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            device
                .device
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())?;
            device
                .device
                .begin_command_buffer(self.command_buffer, &begin_info)?;

            device.device.cmd_copy_buffer(
                self.command_buffer,
                self.staging_buffer.buffer,
                self.vertex_buffer.buffer,
                &[vertex_copy],
            );

            device.device.cmd_copy_buffer(
                self.command_buffer,
                self.staging_buffer.buffer,
                self.index_buffer.buffer,
                &[index_copy],
            );

            device.device.end_command_buffer(self.command_buffer)?;
        }

        let command_buffer_submit_info =
            vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&self.command_buffer));

        unsafe {
            device.device.reset_fences(&[self.upload_fence])?;

            device.device.queue_submit(
                device.graphics_queue,
                &[command_buffer_submit_info],
                self.upload_fence,
            )?;

            device
                .device
                .wait_for_fences(&[self.upload_fence], true, u64::MAX)?;
        }

        let mesh = Mesh::new(index_size, self.index_offset, self.vertex_offset);

        self.vertex_offset += vertex_size as u64;
        self.index_offset += index_size as u64;

        self.meshes.push(mesh);
        Ok(MeshHandle(self.meshes.len() - 1))
    }

    fn copy_to_staging<T>(
        &mut self,
        allocator: &VulkanAllocator,
        data: &[T],
        offset: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mapped = unsafe {
            allocator
                .allocator
                .map_memory(&mut self.staging_buffer.allocation)
        }?;

        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const u8,
                mapped.add(offset),
                size_of::<T>() * data.len(),
            );
        };

        unsafe {
            allocator
                .allocator
                .unmap_memory(&mut self.staging_buffer.allocation);
        }

        Ok(())
    }

    pub fn destroy(&mut self, allocator: &VulkanAllocator) {
        self.vertex_buffer.destroy(allocator);
        self.index_buffer.destroy(allocator);
        self.staging_buffer.destroy(allocator);
    }
}
