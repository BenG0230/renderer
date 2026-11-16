use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
};

use glam::{Vec2, Vec3};

use crate::vulkan::{
    models::{
        mesh_manager::{MeshHandle, MeshManager},
        model::Model,
    },
    renderer::Vertex,
};

pub struct ModelHandle {
    index: usize,
}

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

    pub fn load_model(&mut self, path: &Path) -> Result<ModelHandle, Box<dyn Error>> {
        let path = &path.canonicalize()?;

        if let Some(handle) = self.loaded_models.get(path) {
            return Ok(*handle);
        }

        let model_data = Self::load_model_data(path)?;
    }

    fn load_model_data(
        path: &PathBuf,
        mesh_manager: &mut MeshManager,
    ) -> Result<(), Box<dyn Error>> {
        let mesh_handles: Vec<MeshHandle> = Vec::new();
        let (models, _materials) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;

        for (_, m) in models.iter().enumerate() {
            let mesh = &m.mesh;

            let mut vertices: Vec<Vertex> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();

            for vi in 0..mesh.positions.len() / 3 {
                let vertex_normal_index = vi * 3;
                let uv_index = vi * 2;
                vertices.push(Vertex {
                    pos: Vec3::new(
                        mesh.positions[vertex_normal_index],
                        mesh.positions[vertex_normal_index + 1],
                        mesh.positions[vertex_normal_index + 2],
                    ),
                    normal: Vec3::new(
                        mesh.normals[vertex_normal_index],
                        mesh.normals[vertex_normal_index + 1],
                        mesh.normals[vertex_normal_index + 2],
                    ),
                    uv: Vec2::new(mesh.texcoords[uv_index], mesh.texcoords[uv_index * 2]),
                })
            }

            for index in &mesh.indices {
                indices.push(*index);
            }

            mesh_manager.load_mesh(vertices, indices)?;
        }

        Ok(())
    }
}
