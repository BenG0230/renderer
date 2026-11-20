use std::error::Error;

use ash::vk;

use crate::vulkan::{
    allocation::{Buffer, VulkanAllocator},
    device::Device,
    models::mesh::Mesh,
    renderer::Vertex,
    sync::create_fence,
};

const VERTEX_BUFFER_INITIAL_SIZE: u64 = 128 * 1024 * 1024;
const INDEX_BUFFER_INITIAL_SIZE: u64 = 16 * 1024 * 1024;

const BUFFER_MAX_SIZE: u64 = 512 * 1024 * 1024;

const STAGING_BUFFER_SIZE: u64 = 16 * 1024 * 1024;

pub struct MeshHandle(usize);

pub struct MeshManager {
    meshes: Vec<Mesh>,

    command_buffer: vk::CommandBuffer,
    upload_fence: vk::Fence,

    vertex_buffer: Buffer,
    vertex_buffer_size: u64,
    vertex_offset: u64,

    index_buffer: Buffer,
    index_buffer_size: u64,
    index_offset: u64,

    staging_buffer: Buffer,
}

impl MeshManager {
    pub fn new(
        allocator: &VulkanAllocator,
        device: &Device,
        command_buffer: vk::CommandBuffer,
    ) -> Result<Self, Box<dyn Error>> {
        let vertex_buffer = Buffer::vertex_buffer(allocator, VERTEX_BUFFER_INITIAL_SIZE)?;
        let index_buffer = Buffer::index_buffer(allocator, INDEX_BUFFER_INITIAL_SIZE)?;

        let staging_buffer = Buffer::staging_buffer(allocator, STAGING_BUFFER_SIZE)?;

        Ok(Self {
            meshes: Vec::new(),

            command_buffer,
            upload_fence: create_fence(device)?,

            vertex_buffer,
            vertex_buffer_size: VERTEX_BUFFER_INITIAL_SIZE,
            vertex_offset: 0,

            index_buffer,
            index_buffer_size: INDEX_BUFFER_INITIAL_SIZE,
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

        if vertex_size as u64 + self.vertex_offset > self.vertex_buffer_size {
            log::warn!("Too many vertices! resizing vertex buffer");

            let mut new_size = self.vertex_buffer_size;
            while new_size < vertex_size as u64 + self.vertex_offset {
                new_size *= 2;
            }

            self.resize_vertex_buffer(allocator, device, self.vertex_buffer_size, new_size)?;
            self.vertex_buffer_size = new_size;
        }
        if index_size as u64 + self.index_offset > self.index_buffer_size {
            log::warn!("Too many indices! resizing index buffer");

            let mut new_size = self.vertex_buffer_size;
            while new_size < vertex_size as u64 + self.vertex_offset {
                new_size *= 2;
            }

            self.resize_index_buffer(allocator, device, self.vertex_buffer_size, new_size)?;
            self.index_buffer_size = new_size;
        }

        log::debug!(
            "Mesh size: vertex - {:.2}MiB index - {:.2}MiB",
            vertex_size as f64 / (1024.0 * 1024.0),
            index_size as f64 / (1024.0 * 1024.0)
        );

        if vertex_size + index_size < STAGING_BUFFER_SIZE as usize {
            self.load_mesh_all(allocator, device, vertices, indices)
        } else {
            self.load_mesh_chunks(allocator, device, vertices, indices)
        }?;

        let mesh = Mesh::new(
            indices.len(),
            (self.index_offset / size_of::<u32>() as u64) as u32,
            (self.vertex_offset / size_of::<Vertex>() as u64) as i32,
        );

        self.vertex_offset += vertex_size as u64;
        self.index_offset += index_size as u64;

        self.meshes.push(mesh);
        Ok(MeshHandle(self.meshes.len() - 1))
    }

    fn load_mesh_chunks(
        &mut self,
        allocator: &VulkanAllocator,
        device: &Device,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<(), Box<dyn Error>> {
        let vertex_bytes = as_bytes(vertices);
        let index_bytes = as_bytes(indices);

        for (i, chunk) in vertex_bytes
            .chunks(STAGING_BUFFER_SIZE as usize)
            .enumerate()
        {
            self.copy_chunk(
                allocator,
                device,
                self.vertex_buffer.buffer,
                self.vertex_offset + (i as u64 * STAGING_BUFFER_SIZE),
                chunk,
            )?;
        }

        for (i, chunk) in index_bytes.chunks(STAGING_BUFFER_SIZE as usize).enumerate() {
            self.copy_chunk(
                allocator,
                device,
                self.index_buffer.buffer,
                self.index_offset + (i as u64 * STAGING_BUFFER_SIZE),
                chunk,
            )?;
        }
        Ok(())
    }

    fn copy_chunk<T>(
        &mut self,
        allocator: &VulkanAllocator,
        device: &Device,
        dst_buffer: vk::Buffer,
        offset: u64,
        chunk: &[T],
    ) -> Result<(), Box<dyn Error>> {
        self.copy_to_staging(allocator, chunk, 0)?;

        let copy = vk::BufferCopy::default()
            .src_offset(0)
            .dst_offset(offset)
            .size(STAGING_BUFFER_SIZE.min(chunk.len() as u64));

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
                dst_buffer,
                &[copy],
            );

            device.device.end_command_buffer(self.command_buffer)?;

            let command_buffer_submit_info = vk::SubmitInfo::default()
                .command_buffers(std::slice::from_ref(&self.command_buffer));

            device.device.reset_fences(&[self.upload_fence])?;
            device.device.queue_submit(
                device.graphics_queue,
                &[command_buffer_submit_info],
                self.upload_fence,
            )?;
            device
                .device
                .wait_for_fences(&[self.upload_fence], true, u64::MAX)?;
        };

        Ok(())
    }

    fn load_mesh_all(
        &mut self,
        allocator: &VulkanAllocator,
        device: &Device,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<(), Box<dyn Error>> {
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
        Ok(())
    }

    fn copy_to_staging<T>(
        &mut self,
        allocator: &VulkanAllocator,
        data: &[T],
        offset: usize,
    ) -> Result<(), Box<dyn Error>> {
        unsafe {
            let mapped = allocator
                .allocator
                .map_memory(&mut self.staging_buffer.allocation)?;

            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const u8,
                mapped.add(offset),
                size_of::<T>() * data.len(),
            );

            allocator
                .allocator
                .unmap_memory(&mut self.staging_buffer.allocation);
        };

        Ok(())
    }

    fn resize_vertex_buffer(
        &mut self,
        allocator: &VulkanAllocator,
        device: &Device,
        old_size: u64,
        new_size: u64,
    ) -> Result<(), Box<dyn Error>> {
        if new_size > BUFFER_MAX_SIZE {
            return Err(format!(
                "Vertex buffer to grow too big! {}mb",
                new_size / (1024 * 1024)
            )
            .into());
        }

        let new_buffer = Buffer::vertex_buffer(allocator, new_size)?;

        let copy = vk::BufferCopy::default()
            .src_offset(0)
            .dst_offset(0)
            .size(old_size);

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
                self.vertex_buffer.buffer,
                new_buffer.buffer,
                &[copy],
            );

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
        }

        self.vertex_buffer.destroy(allocator);

        self.vertex_buffer = new_buffer;

        log::debug!("Vertex buffer resized to {}MiB", new_size / (1024 * 1024));

        Ok(())
    }

    fn resize_index_buffer(
        &mut self,
        allocator: &VulkanAllocator,
        device: &Device,
        old_size: u64,
        new_size: u64,
    ) -> Result<(), Box<dyn Error>> {
        if new_size > BUFFER_MAX_SIZE {
            return Err(format!(
                "Vertex buffer to grow too big! {}mb",
                new_size / (1024 * 1024)
            )
            .into());
        }

        let new_buffer = Buffer::index_buffer(allocator, new_size)?;

        let copy = vk::BufferCopy::default()
            .src_offset(0)
            .dst_offset(0)
            .size(old_size);

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
                self.index_buffer.buffer,
                new_buffer.buffer,
                &[copy],
            );

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
        }

        self.index_buffer.destroy(allocator);

        self.index_buffer = new_buffer;

        log::debug!("Index buffer resized to {}MiB", new_size / (1024 * 1024));

        Ok(())
    }

    pub fn get_vertex_buffer(&self) -> vk::Buffer {
        self.vertex_buffer.buffer
    }

    pub fn get_index_buffer(&self) -> vk::Buffer {
        self.index_buffer.buffer
    }

    pub fn get_mesh(&self, handle: &MeshHandle) -> &Mesh {
        &self.meshes[handle.0]
    }

    pub fn destroy(&mut self, device: &Device, allocator: &VulkanAllocator) {
        self.vertex_buffer.destroy(allocator);
        self.index_buffer.destroy(allocator);
        self.staging_buffer.destroy(allocator);

        unsafe { device.device.destroy_fence(self.upload_fence, None) };
    }
}

fn as_bytes<T>(slice: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, size_of::<T>() * slice.len()) }
}
