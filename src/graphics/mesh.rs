#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Vertex {
    // TODO: check alignment.
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
    tangent: [f32; 4],
    // TODO: declare vertex data fields.
}

impl Vertex {
    pub fn new(
        position: [f32; 3],
        tex_coords: [f32; 2],
        normal: [f32; 3],
        tangent: [f32; 4],
    ) -> Self {
        Self {
            position,
            tex_coords,
            normal,
            tangent,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Mesh {
    // TODO: check alignment.
    vertex_offset: u32,
    vertex_count: u32,
    index_offset: u32,
    index_count: u32,
}

impl Mesh {
    pub fn new(vertex_offset: u32, vertex_count: u32, index_offset: u32, index_count: u32) -> Self {
        Self {
            vertex_offset,
            vertex_count,
            index_offset,
            index_count,
        }
    }
}
