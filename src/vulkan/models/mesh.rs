pub struct Mesh {
    index_count: usize,
    first_index: u64,

    vertex_offset: u64,
}

impl Mesh {
    pub fn new(index_count: usize, first_index: u64, vertex_offset: u64) -> Self {
        Self {
            index_count,
            first_index,
            vertex_offset,
        }
    }
}
