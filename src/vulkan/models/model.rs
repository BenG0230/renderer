use crate::vulkan::models::mesh_manager::MeshHandle;

pub struct Model {
    pub mesh_handles: Vec<MeshHandle>,
}

impl Model {
    pub fn new(mesh_handles: Vec<MeshHandle>) -> Self {
        Self { mesh_handles }
    }
}
