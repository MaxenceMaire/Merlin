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

                    let gltf_bounding_box = primitive.bounding_box();
                    let bounding_box = graphics::BoundingBox {
                        min: gltf_bounding_box.min,
                        max: gltf_bounding_box.max,
                    };

                    let mesh_index = self.mesh_map.push(
                        format!("{}#{}", canonicalized_path.to_str().unwrap(), name),
                        vertices,
                        indices,
                        bounding_box,
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
    pub bounding_boxes: Vec<graphics::BoundingBox>,
    pub map: HashMap<String, u32>,
}

impl MeshMap {
    pub fn push(
        &mut self,
        name: String,
        vertices: Vec<graphics::Vertex>,
        indices: Vec<u32>,
        bounding_box: graphics::BoundingBox,
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
        self.bounding_boxes.push(bounding_box);
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
    RgBc5Unorm512,
    RgBc5Unorm1024,
    RgBc5Unorm2048,
    RgBc5Unorm4096,
    RgbBc7Unorm512,
    RgbBc7Unorm1024,
    RgbBc7Unorm2048,
    RgbBc7Unorm4096,
    RgbaBc7Srgb512,
    RgbaBc7Srgb1024,
    RgbaBc7Srgb2048,
    RgbaBc7Srgb4096,
}

impl TextureArray {
    pub fn id(&self) -> u32 {
        match self {
            Self::RgBc5Unorm512 => 0,
            Self::RgBc5Unorm1024 => 1,
            Self::RgBc5Unorm2048 => 2,
            Self::RgBc5Unorm4096 => 3,
            Self::RgbBc7Unorm512 => 4,
            Self::RgbBc7Unorm1024 => 5,
            Self::RgbBc7Unorm2048 => 6,
            Self::RgbBc7Unorm4096 => 7,
            Self::RgbaBc7Srgb512 => 8,
            Self::RgbaBc7Srgb1024 => 9,
            Self::RgbaBc7Srgb2048 => 10,
            Self::RgbaBc7Srgb4096 => 11,
        }
    }
}

pub struct TextureArrays {
    pub rg_bc5_unorm_512: TextureMap,
    pub rg_bc5_unorm_1024: TextureMap,
    pub rg_bc5_unorm_2048: TextureMap,
    pub rg_bc5_unorm_4096: TextureMap,
    pub rgb_bc7_unorm_512: TextureMap,
    pub rgb_bc7_unorm_1024: TextureMap,
    pub rgb_bc7_unorm_2048: TextureMap,
    pub rgb_bc7_unorm_4096: TextureMap,
    pub rgba_bc7_srgb_512: TextureMap,
    pub rgba_bc7_srgb_1024: TextureMap,
    pub rgba_bc7_srgb_2048: TextureMap,
    pub rgba_bc7_srgb_4096: TextureMap,
}

impl TextureArrays {
    pub fn new() -> Self {
        Self {
            rg_bc5_unorm_512: Default::default(),
            rg_bc5_unorm_1024: Default::default(),
            rg_bc5_unorm_2048: Default::default(),
            rg_bc5_unorm_4096: Default::default(),
            rgb_bc7_unorm_512: Default::default(),
            rgb_bc7_unorm_1024: Default::default(),
            rgb_bc7_unorm_2048: Default::default(),
            rgb_bc7_unorm_4096: Default::default(),
            rgba_bc7_srgb_512: Default::default(),
            rgba_bc7_srgb_1024: Default::default(),
            rgba_bc7_srgb_2048: Default::default(),
            rgba_bc7_srgb_4096: Default::default(),
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
            (Some(ktx2::Format::BC5_UNORM_BLOCK), 512, 512) => (
                TextureArray::RgBc5Unorm512,
                self.rg_bc5_unorm_512.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::BC5_UNORM_BLOCK), 1024, 1024) => (
                TextureArray::RgBc5Unorm1024,
                self.rg_bc5_unorm_1024.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::BC5_UNORM_BLOCK), 2048, 2048) => (
                TextureArray::RgBc5Unorm2048,
                self.rg_bc5_unorm_2048.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::BC5_UNORM_BLOCK), 4096, 4096) => (
                TextureArray::RgBc5Unorm4096,
                self.rg_bc5_unorm_4096.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::BC7_UNORM_BLOCK), 512, 512) => (
                TextureArray::RgbBc7Unorm512,
                self.rgb_bc7_unorm_512.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::BC7_UNORM_BLOCK), 1024, 1024) => (
                TextureArray::RgbBc7Unorm1024,
                self.rgb_bc7_unorm_1024.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::BC7_UNORM_BLOCK), 2048, 2048) => (
                TextureArray::RgbBc7Unorm2048,
                self.rgb_bc7_unorm_2048.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::BC7_UNORM_BLOCK), 4096, 4096) => (
                TextureArray::RgbBc7Unorm4096,
                self.rgb_bc7_unorm_4096.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::BC7_SRGB_BLOCK), 512, 512) => (
                TextureArray::RgbaBc7Srgb512,
                self.rgba_bc7_srgb_512.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::BC7_SRGB_BLOCK), 1024, 1024) => (
                TextureArray::RgbaBc7Srgb1024,
                self.rgba_bc7_srgb_1024.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::BC7_SRGB_BLOCK), 2048, 2048) => (
                TextureArray::RgbaBc7Srgb2048,
                self.rgba_bc7_srgb_2048.add(name, texture.data().to_vec()),
            ),
            (Some(ktx2::Format::BC7_SRGB_BLOCK), 4096, 4096) => (
                TextureArray::RgbaBc7Srgb4096,
                self.rgba_bc7_srgb_4096.add(name, texture.data().to_vec()),
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
