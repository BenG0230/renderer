use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
};

use glam::{Vec2, Vec3};

use crate::vulkan::{
    allocation::VulkanAllocator,
    device::Device,
    models::{
        mesh_manager::{MeshHandle, MeshManager},
        model::Model,
    },
    renderer::Vertex,
};

#[derive(Clone, Copy)]
pub struct ModelHandle(usize);

pub struct ModelManager {
    models: Vec<Model>,
    loaded_models: HashMap<PathBuf, ModelHandle>,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            loaded_models: HashMap::new(),
        }
    }

    pub fn load_model(
        &mut self,
        path: &Path,
        mesh_manager: &mut MeshManager,
        allocator: &VulkanAllocator,
        device: &Device,
    ) -> Result<ModelHandle, Box<dyn Error>> {
        let path = &path.canonicalize()?;

        if let Some(handle) = self.loaded_models.get(path) {
            return Ok(*handle);
        }

        let model = Self::load_model_data(path, mesh_manager, allocator, device)?;

        let handle = ModelHandle(self.models.len());
        self.models.push(model);
        self.loaded_models.insert(path.to_path_buf(), handle);

        Ok(handle)
    }

    fn load_model_data(
        path: &PathBuf,
        mesh_manager: &mut MeshManager,
        allocator: &VulkanAllocator,
        device: &Device,
    ) -> Result<Model, Box<dyn Error>> {
        let mut mesh_handles: Vec<MeshHandle> = Vec::new();
        let (models, _materials) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;

        for (_, m) in models.iter().enumerate() {
            let mesh = &m.mesh;

            let mut vertices: Vec<Vertex> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();

            for vi in 0..mesh.positions.len() / 3 {
                let vertex_normal_index = vi * 3;
                let uv_index = vi * 2;

                let pos = Vec3::new(
                    mesh.positions[vertex_normal_index],
                    mesh.positions[vertex_normal_index + 1],
                    mesh.positions[vertex_normal_index + 2],
                );

                let normal = if mesh.normals.len() != 0 {
                    Vec3::new(
                        mesh.normals[vertex_normal_index],
                        mesh.normals[vertex_normal_index + 1],
                        mesh.normals[vertex_normal_index + 2],
                    )
                } else {
                    Vec3::ZERO
                };

                let uv = if mesh.texcoords.len() != 0 {
                    Vec2::new(mesh.texcoords[uv_index], mesh.texcoords[uv_index + 1])
                } else {
                    Vec2::ZERO
                };

                vertices.push(Vertex { pos, normal, uv })
            }

            for index in &mesh.indices {
                indices.push(*index);
            }

            mesh_handles.push(mesh_manager.load_mesh(allocator, device, &vertices, &indices)?);
        }

        let model = Model::new(mesh_handles);

        Ok(model)
    }

    pub fn get_models(&self) -> &Vec<Model> {
        &self.models
    }
}
