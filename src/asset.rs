use crate::graphics;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

pub type TextureArrayId = u32;
pub type TextureId = u32;
pub type MeshId = u32;
pub type MaterialId = u32;
pub type ModelId = usize;

pub struct AssetLoader {
    pub mesh_map: MeshMap,
    pub texture_arrays: TextureArrays,
    pub texture_dictionary: HashMap<String, (TextureArrayId, TextureId)>,
    pub material_map: MaterialMap,
    pub model_map: ModelMap,
}

impl AssetLoader {
    pub fn new() -> Self {
        Self {
            mesh_map: MeshMap::default(),
            texture_arrays: TextureArrays::new(),
            texture_dictionary: HashMap::new(),
            material_map: MaterialMap::default(),
            model_map: ModelMap::default(),
        }
    }

    pub fn load_gltf_model<P>(&mut self, path: P) -> Result<ModelId, GltfError>
    where
        P: AsRef<std::path::Path>,
    {
        // TODO: remove assets path prefix.

        let path = path.as_ref();
        let canonicalized_path = path.canonicalize().unwrap();

        if let Some(&model_id) = self.model_map.map.get(canonicalized_path.to_str().unwrap()) {
            return Ok(model_id);
        }

        // TODO: error message. No parent path.
        let directory_path = path.parent().unwrap();

        let mut load_texture =
            |texture_path: &std::path::Path| -> Result<(TextureArrayId, TextureId), GltfError> {
                let texture_path_str = texture_path.to_str().unwrap();

                let (texture_array_id, texture_id) = if let Some((texture_array_id, texture_id)) =
                    self.texture_dictionary.get(texture_path_str)
                {
                    (*texture_array_id, *texture_id)
                } else {
                    let texture_data = std::fs::read(texture_path_str).unwrap();
                    let texture_reader = ktx2::Reader::new(&texture_data).unwrap();
                    let (format, texture_id) = self
                        .texture_arrays
                        .add(texture_path_str.to_string(), texture_reader)?;
                    let texture_array_id = format.id();
                    self.texture_dictionary
                        .insert(texture_path_str.to_string(), (texture_array_id, texture_id));
                    (texture_array_id, texture_id)
                };

                Ok((texture_array_id, texture_id))
            };

        let gltf::Gltf { document, .. } = gltf::Gltf::open(path).unwrap();
        let buffers = gltf::import_buffers(&document, Some(directory_path), None).unwrap();
        let mut nodes = Vec::new();
        let mut stack = VecDeque::new();

        let default_scene = document.default_scene().unwrap();
        for gltf_node in default_scene.nodes() {
            stack.push_back(gltf_node);
        }

        while let Some(gltf_node) = stack.pop_front() {
            let mut node = Node {
                name: gltf_node.name().map(|s| s.to_string()),
                object_group: None,
                children: Vec::new(),
            };

            if let Some(gltf_mesh) = gltf_node.mesh() {
                let mut object_group = ObjectGroup {
                    objects: Vec::new(),
                };

                for (i, primitive) in gltf_mesh.primitives().enumerate() {
                    let name = format!("{}/{}", gltf_mesh.name().unwrap(), i);
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                    let gltf_material = primitive.material();
                    let pbr_metallic_roughness = gltf_material.pbr_metallic_roughness();
                    // TODO: error message. Object must have a base color texture.
                    let base_color_texture_information =
                        pbr_metallic_roughness.base_color_texture().unwrap();

                    let (base_color_texture_array_id, base_color_texture_id) =
                        if let gltf::image::Source::Uri { uri, .. } =
                            base_color_texture_information.texture().source().source()
                        {
                            // TODO: error message. error if invalid file path.
                            let texture_path = directory_path.join(uri).canonicalize().unwrap();
                            load_texture(&texture_path)?
                        } else {
                            // TODO: error message. Expected URI source.
                            panic!();
                        };

                    let tex_coords_set_index = base_color_texture_information.tex_coord();
                    // TODO: error message.
                    let tex_coords = if let gltf::mesh::util::ReadTexCoords::F32(tex_coords_iter) =
                        reader.read_tex_coords(tex_coords_set_index).unwrap()
                    {
                        tex_coords_iter
                    } else {
                        // TODO: error message. Expected f32 tex coords.
                        panic!();
                    };

                    let (normal_texture_array_id, normal_texture_id) =
                        if let gltf::image::Source::Uri { uri, .. } = gltf_material
                            .normal_texture()
                            .unwrap()
                            .texture()
                            .source()
                            .source()
                        {
                            // TODO: error message. error if invalid file path.
                            let texture_path = directory_path.join(uri).canonicalize().unwrap();
                            load_texture(&texture_path)?
                        } else {
                            // TODO: error message. Expected URI source.
                            panic!();
                        };

                    let material = graphics::Material::new(
                        base_color_texture_array_id,
                        base_color_texture_id,
                        normal_texture_array_id,
                        normal_texture_id,
                    );

                    let material_index = self.material_map.add(material);

                    // TODO: add metallic and roughness information to material.
                    let metallic_roughness_texture_information =
                        pbr_metallic_roughness.metallic_roughness_texture().unwrap();

                    let vertex_positions = reader.read_positions().unwrap(); // TODO: error message.
                    let vertex_normals = reader.read_normals().unwrap(); // TODO: error message.
                    let vertex_tangents = reader.read_tangents().unwrap(); // TODO: error message.

                    let mut vertices = Vec::with_capacity(vertex_positions.len());
                    for (position, tex_coord, normal, tangent) in itertools::izip!(
                        vertex_positions,
                        tex_coords,
                        vertex_normals,
                        vertex_tangents
                    ) {
                        let bitangent = glam::Vec3::from(normal)
                            .cross(glam::Vec3::new(tangent[0], tangent[1], tangent[2]))
                            * tangent[3];

                        vertices.push(graphics::Vertex::new(
                            position,
                            tex_coord,
                            normal,
                            tangent,
                            bitangent.into(),
                        ));
                    }

                    let indices = reader
                        .read_indices()
                        .unwrap() // TODO: error message.
                        .into_u32()
                        .collect::<Vec<_>>();

                    let mesh_index = self.mesh_map.push(
                        format!("{}#{}", canonicalized_path.to_str().unwrap(), name),
                        vertices,
                        indices,
                    );

                    object_group.objects.push(Object {
                        mesh_id: mesh_index,
                        material_id: material_index,
                    });
                }

                node.object_group = Some(object_group);
            }

            for child in gltf_node.children() {
                stack.push_back(child);
                node.children.push(nodes.len());
            }

            nodes.push(node);
        }

        let model = Model {
            root_nodes: Vec::from_iter(0..default_scene.nodes().len()),
            nodes,
        };

        let model_index = self
            .model_map
            .add(path.to_str().unwrap().to_string(), model);

        Ok(model_index)
    }
}

#[derive(Debug)]
pub struct Model {
    pub root_nodes: Vec<usize>,
    pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub struct Node {
    pub name: Option<String>,
    pub object_group: Option<ObjectGroup>,
    pub children: Vec<usize>,
}

#[derive(Debug)]
pub struct Object {
    pub mesh_id: MeshId,
    pub material_id: MaterialId,
}

#[derive(Debug)]
pub struct ObjectGroup {
    pub objects: Vec<Object>,
}

#[derive(Default)]
pub struct ModelMap {
    pub models: Vec<Model>,
    pub map: HashMap<String, usize>,
}

impl ModelMap {
    pub fn add(&mut self, name: String, model: Model) -> usize {
        if let Some(&model_index) = self.map.get(&name) {
            return model_index;
        }

        let model_index = self.models.len();

        self.models.push(model);
        self.map.insert(name, model_index);

        model_index
    }

    pub fn index(&self, i: usize) -> Option<&Model> {
        self.models.get(i)
    }

    pub fn get(&self, name: &str) -> Option<&Model> {
        self.map
            .get(name)
            .map(|&model_index| &self.models[model_index])
    }
}

#[derive(Default, Debug)]
pub struct MeshMap {
    pub vertices: Vec<graphics::Vertex>,
    pub indices: Vec<u32>,
    pub meshes: Vec<graphics::Mesh>,
    pub map: HashMap<String, u32>,
}

impl MeshMap {
    pub fn push(
        &mut self,
        name: String,
        vertices: Vec<graphics::Vertex>,
        indices: Vec<u32>,
    ) -> u32 {
        if let Some(&mesh_index) = self.map.get(&name) {
            return mesh_index;
        }

        let vertex_offset = self.vertices.len() as u32;
        let vertex_count = vertices.len() as u32;
        let index_offset = self.indices.len() as u32;
        let index_count = indices.len() as u32;

        let mesh_index = self.meshes.len() as u32;

        self.meshes.push(graphics::Mesh::new(
            vertex_offset,
            vertex_count,
            index_offset,
            index_count,
        ));
        self.vertices.extend(vertices);
        self.indices.extend(indices);
        self.map.insert(name, mesh_index);

        mesh_index
    }
}

#[derive(Default, Debug)]
pub struct TextureMap {
    pub textures: graphics::TextureArray,
    pub map: HashMap<String, u32>,
}

impl TextureMap {
    pub fn add(&mut self, name: String, texture: Vec<u8>) -> u32 {
        if let Some(&texture_index) = self.map.get(&name) {
            return texture_index;
        }

        let texture_index = self.map.len() as u32;

        self.textures.extend(texture);
        self.map.insert(name, texture_index);

        texture_index
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum TextureArray {
    UnormSrgb512,
    UnormSrgb1024,
    UnormSrgb2048,
    UnormSrgb4096,
    Unorm512,
    Unorm1024,
    Unorm2048,
    Unorm4096,
    Hdr512,
    Hdr1024,
    Hdr2048,
    Hdr4096,
}

impl TextureArray {
    pub fn id(&self) -> u32 {
        match self {
            Self::UnormSrgb512 => 0,
            Self::UnormSrgb1024 => 1,
            Self::UnormSrgb2048 => 2,
            Self::UnormSrgb4096 => 3,
            Self::Unorm512 => 4,
            Self::Unorm1024 => 5,
            Self::Unorm2048 => 6,
            Self::Unorm4096 => 7,
            Self::Hdr512 => 8,
            Self::Hdr1024 => 9,
            Self::Hdr2048 => 10,
            Self::Hdr4096 => 11,
        }
    }
}

pub struct TextureArrays {
    pub unorm_srgb_512: TextureMap,
    pub unorm_srgb_1024: TextureMap,
    pub unorm_srgb_2048: TextureMap,
    pub unorm_srgb_4096: TextureMap,
    pub unorm_512: TextureMap,
    pub unorm_1024: TextureMap,
    pub unorm_2048: TextureMap,
    pub unorm_4096: TextureMap,
    pub hdr_512: TextureMap,
    pub hdr_1024: TextureMap,
    pub hdr_2048: TextureMap,
    pub hdr_4096: TextureMap,
}

impl TextureArrays {
    pub fn new() -> Self {
        Self {
            unorm_srgb_512: TextureMap::default(),
            unorm_srgb_1024: TextureMap::default(),
            unorm_srgb_2048: TextureMap::default(),
            unorm_srgb_4096: TextureMap::default(),
            unorm_512: TextureMap::default(),
            unorm_1024: TextureMap::default(),
            unorm_2048: TextureMap::default(),
            unorm_4096: TextureMap::default(),
            hdr_512: TextureMap::default(),
            hdr_1024: TextureMap::default(),
            hdr_2048: TextureMap::default(),
            hdr_4096: TextureMap::default(),
        }
    }

    pub fn add(
        &mut self,
        name: String,
        texture: ktx2::Reader<&Vec<u8>>,
    ) -> Result<(TextureArray, u32), GltfError> {
        let ktx2::Header {
            format,
            pixel_width: width,
            pixel_height: height,
            ..
        } = texture.header();

        let (texture_array, texture_index) = match (format, width, height) {
            (Some(ktx2::Format::ASTC_4x4_SRGB_BLOCK), 512, 512) => (
                TextureArray::UnormSrgb512,
                self.unorm_srgb_512.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::ASTC_4x4_SRGB_BLOCK), 1024, 1024) => (
                TextureArray::UnormSrgb1024,
                self.unorm_srgb_1024.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::ASTC_4x4_SRGB_BLOCK), 2048, 2048) => (
                TextureArray::UnormSrgb2048,
                self.unorm_srgb_2048.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::ASTC_4x4_SRGB_BLOCK), 4096, 4096) => (
                TextureArray::UnormSrgb4096,
                self.unorm_srgb_4096.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::ASTC_4x4_UNORM_BLOCK), 512, 512) => (
                TextureArray::Unorm512,
                self.unorm_srgb_512.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::ASTC_4x4_UNORM_BLOCK), 1024, 1024) => (
                TextureArray::Unorm1024,
                self.unorm_srgb_1024.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::ASTC_4x4_UNORM_BLOCK), 2048, 2048) => (
                TextureArray::Unorm2048,
                self.unorm_srgb_2048.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::ASTC_4x4_UNORM_BLOCK), 4096, 4096) => (
                TextureArray::Unorm4096,
                self.unorm_srgb_4096.add(name, texture.data().to_vec()),
            ),
            _ => {
                return Err(GltfError::UnsupportedTextureFormat {
                    name,
                    format,
                    width,
                    height,
                });
            }
        };

        Ok((texture_array, texture_index))
    }
}

#[derive(Default, Debug)]
pub struct MaterialMap {
    pub materials: Vec<graphics::Material>,
    pub map: HashMap<graphics::Material, u32>,
}

impl MaterialMap {
    pub fn add(&mut self, material: graphics::Material) -> u32 {
        if let Some(&material_index) = self.map.get(&material) {
            return material_index;
        }

        let material_index = self.materials.len() as u32;

        self.materials.push(material);
        self.map.insert(material, material_index);

        material_index
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum GltfError {
    InvalidTexture {
        name: String,
    },
    UnsupportedTextureFormat {
        name: String,
        format: Option<ktx2::Format>,
        width: u32,
        height: u32,
    },
}

impl std::error::Error for GltfError {}

impl std::fmt::Display for GltfError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Self::InvalidTexture { name } => write!(f, "invalid texture with name \"{name}\""),
            Self::UnsupportedTextureFormat {
                name,
                format,
                width,
                height,
            } => {
                write!(
                    f,
                    "unsupported texture format {:?} with size {width}*{height} for \"{name}\"",
                    format
                )
            }
        }
    }
}
