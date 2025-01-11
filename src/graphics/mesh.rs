#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Vertex {
    pub fn new(
        position: [f32; 3],
        tex_coords: [f32; 2],
        normal: [f32; 3],
        tangent: [f32; 3],
        bitangent: [f32; 3],
    ) -> Self {
        Self {
            position,
            tex_coords,
            normal,
            tangent,
            bitangent,
        }
    }

    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Mesh {
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub index_offset: u32,
    pub index_count: u32,
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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct BoundingBox {
    pub min: [f32; 3],
    _padding_0: f32,
    pub max: [f32; 3],
    _padding_1: f32,
}

impl BoundingBox {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self {
            min,
            _padding_0: 0.0,
            max,
            _padding_1: 0.0,
        }
    }
}

pub mod primitive {
    use super::*;

    #[derive(Debug)]
    pub struct Icosphere {
        pub vertices: Vec<Vertex>,
        pub indices: Vec<u32>,
        subdivision_level: usize,
    }

    impl Icosphere {
        pub fn with_subdivision_level(subdivision_level: usize) -> Self {
            const PHI: f32 = 1.618_034;

            assert!(
                (1..=20).contains(&subdivision_level),
                "icosphere subdivision level must be inside [1;20]"
            );

            let mut vertices: Vec<Vertex> = Vec::new();

            let icosahedron_vertex_positions = [
                [-1.0, PHI, 0.0],
                [1.0, PHI, 0.0],
                [-1.0, -PHI, 0.0],
                [1.0, -PHI, 0.0],
                [0.0, -1.0, PHI],
                [0.0, 1.0, PHI],
                [0.0, -1.0, -PHI],
                [0.0, 1.0, -PHI],
                [PHI, 0.0, -1.0],
                [PHI, 0.0, 1.0],
                [-PHI, 0.0, -1.0],
                [-PHI, 0.0, 1.0],
            ]
            .map(|vertex_position| glam::Vec3::from(vertex_position).normalize().to_array());

            let vertex = |position: glam::Vec3| {
                let normal = position;
                let arbitrary_vector =
                    if normal.x.abs() < normal.y.abs() && normal.x.abs() < normal.z.abs() {
                        // X-axis is least aligned with the normal.
                        glam::Vec3::new(1.0, 0.0, 0.0)
                    } else {
                        // Y-axis is least aligned with the normal.
                        glam::Vec3::new(0.0, 1.0, 0.0)
                    };
                let tangent = normal.cross(arbitrary_vector).normalize();
                let bitangent = normal.cross(tangent).normalize();

                let azimuthal_angle = normal.z.atan2(normal.x) / 2.0 * std::f32::consts::PI + 0.5;
                let polar_angle = normal.y.asin() / std::f32::consts::PI + 0.5;

                Vertex {
                    position: position.into(),
                    tex_coords: [azimuthal_angle, polar_angle],
                    normal: normal.into(),
                    tangent: tangent.into(),
                    bitangent: bitangent.into(),
                }
            };

            for vertex_position in icosahedron_vertex_positions {
                vertices.push(vertex(vertex_position.into()));
            }

            let mut indices = vec![
                0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 1, 5, 9, 5, 11, 4, 11, 10, 2, 10,
                7, 6, 7, 1, 8, 3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 4, 9, 5, 2, 4, 11, 6,
                2, 10, 8, 6, 7, 9, 8, 1,
            ];

            let mut middle_points: std::collections::HashMap<(u32, u32), u32> =
                std::collections::HashMap::new();

            for _ in 1..subdivision_level {
                let mut new_indices = Vec::new();

                let mut middle_point = |p1, p2| {
                    let key = if p1 < p2 { (p1, p2) } else { (p2, p1) };

                    if let Some(&index) = middle_points.get(&key) {
                        return index;
                    }

                    let pos1 = vertices[p1 as usize].position;
                    let pos2 = vertices[p2 as usize].position;

                    let middle = glam::Vec3::from([
                        (pos1[0] + pos2[0]) / 2.0,
                        (pos1[1] + pos2[1]) / 2.0,
                        (pos1[2] + pos2[2]) / 2.0,
                    ])
                    .normalize();

                    let index = vertices.len() as u32;

                    vertices.push(vertex(middle));

                    middle_points.insert(key, index);

                    index
                };

                for index_chunk in indices.chunks(3) {
                    let middle_point_a = middle_point(index_chunk[0], index_chunk[1]);
                    let middle_point_b = middle_point(index_chunk[1], index_chunk[2]);
                    let middle_point_c = middle_point(index_chunk[2], index_chunk[0]);

                    new_indices.extend_from_slice(&[
                        index_chunk[0],
                        middle_point_a,
                        middle_point_c,
                    ]);
                    new_indices.extend_from_slice(&[
                        index_chunk[1],
                        middle_point_b,
                        middle_point_a,
                    ]);
                    new_indices.extend_from_slice(&[
                        index_chunk[2],
                        middle_point_c,
                        middle_point_b,
                    ]);
                    new_indices.extend_from_slice(&[
                        middle_point_a,
                        middle_point_b,
                        middle_point_c,
                    ]);
                }

                indices = new_indices;
            }

            Self {
                vertices,
                indices,
                subdivision_level,
            }
        }

        pub fn canonic_name(&self) -> String {
            format!("primitive::icosphere_{}", self.subdivision_level)
        }
    }
}
