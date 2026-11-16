use std::error::Error;

use ash::vk;

use crate::vulkan::{models::mesh::Mesh, renderer::Vertex};

pub struct MeshHandle {}

pub struct MeshManager {
    meshes: Vec<Mesh>,

    vertex_buffer: vk::Buffer,
    vertex_buffer_allocator: vk_mem::Allocator,

    index_buffer: vk::Buffer,
    index_buffer_allocator: vk_mem::Allocator,
}

impl MeshManager {
    // pub fn new() -> Self {
    //     Self { meshes: Vec::new() }
    // }

    pub fn load_mesh(
        &mut self,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
