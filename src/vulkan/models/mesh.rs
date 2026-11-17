#[derive(Copy, Clone)]
pub struct Mesh {
    pub index_count: usize,
    pub first_index: u32,

    pub vertex_offset: i32,
}

impl Mesh {
    pub fn new(index_count: usize, first_index: u32, vertex_offset: i32) -> Self {
        Self {
            index_count,
            first_index,
            vertex_offset,
        }
    }
}
