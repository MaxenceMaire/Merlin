use std::collections::HashMap;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Vertex {
    // TODO: check alignment.
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],
    // TODO: declare vertex data fields.
}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], tangent: [f32; 4]) -> Self {
        Self {
            position,
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

#[derive(Default, Debug)]
pub struct MeshMap {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub meshes: Vec<Mesh>,
    pub map: HashMap<String, usize>,
}

impl MeshMap {
    pub fn push(&mut self, name: String, vertices: Vec<Vertex>, indices: Vec<u32>) -> usize {
        if let Some(&mesh_index) = self.map.get(&name) {
            return mesh_index;
        }

        let vertex_offset = self.vertices.len() as u32;
        let vertex_count = vertices.len() as u32;
        let index_offset = self.indices.len() as u32;
        let index_count = indices.len() as u32;

        let mesh_index = self.meshes.len();

        self.meshes.push(Mesh {
            vertex_offset,
            vertex_count,
            index_offset,
            index_count,
        });
        self.vertices.extend(vertices);
        self.indices.extend(indices);
        self.map.insert(name, mesh_index);

        mesh_index
    }
}
