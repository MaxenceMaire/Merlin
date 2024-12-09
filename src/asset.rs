use crate::graphics;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

pub type TextureArrayId = usize;
pub type TextureId = usize;
pub type MeshId = usize;
pub type MaterialId = usize;
pub type ModelId = usize;

pub struct AssetLoader {
    pub mesh_map: MeshMap,
    pub texture_arrays: TextureArrays,
    pub texture_dictionary: HashMap<String, (TextureArrayId, TextureId)>,
    pub material_map: MaterialMap,
    pub model_map: ModelMap,
    pub texture_compression: graphics::gpu::TextureCompression,
}

impl AssetLoader {
    pub fn new(texture_compression: graphics::gpu::TextureCompression) -> Self {
        Self {
            mesh_map: MeshMap::default(),
            texture_arrays: TextureArrays::new(),
            texture_dictionary: HashMap::new(),
            material_map: MaterialMap::default(),
            model_map: ModelMap::default(),
            texture_compression,
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
                    let (format, texture_id) = self.texture_arrays.add(
                        texture_path_str.to_string(),
                        texture_data,
                        self.texture_compression,
                    )?;
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
                        vertices.push(graphics::Vertex::new(position, tex_coord, normal, tangent));
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
    pub map: HashMap<String, usize>,
}

impl MeshMap {
    pub fn push(
        &mut self,
        name: String,
        vertices: Vec<graphics::Vertex>,
        indices: Vec<u32>,
    ) -> usize {
        if let Some(&mesh_index) = self.map.get(&name) {
            return mesh_index;
        }

        let vertex_offset = self.vertices.len() as u32;
        let vertex_count = vertices.len() as u32;
        let index_offset = self.indices.len() as u32;
        let index_count = indices.len() as u32;

        let mesh_index = self.meshes.len();

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
    pub map: HashMap<String, usize>,
}

impl TextureMap {
    pub fn add(&mut self, name: String, texture: Vec<u8>) -> usize {
        if let Some(&texture_index) = self.map.get(&name) {
            return texture_index;
        }

        let texture_index = self.map.len();

        self.textures.extend(texture);
        self.map.insert(name, texture_index);

        texture_index
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum TextureArray {
    Opaque512,
    Opaque1024,
    Opaque2048,
    Opaque4096,
    Transparent512,
    Transparent1024,
    Transparent2048,
    Transparent4096,
}

impl TextureArray {
    pub fn id(&self) -> usize {
        match self {
            Self::Opaque512 => 0,
            Self::Opaque1024 => 1,
            Self::Opaque2048 => 2,
            Self::Opaque4096 => 3,
            Self::Transparent512 => 4,
            Self::Transparent1024 => 5,
            Self::Transparent2048 => 6,
            Self::Transparent4096 => 7,
        }
    }
}

pub struct TextureArrays {
    pub opaque_512: TextureMap,
    pub opaque_1024: TextureMap,
    pub opaque_2048: TextureMap,
    pub opaque_4096: TextureMap,
    pub transparent_512: TextureMap,
    pub transparent_1024: TextureMap,
    pub transparent_2048: TextureMap,
    pub transparent_4096: TextureMap,
}

impl TextureArrays {
    pub fn new() -> Self {
        Self {
            opaque_512: TextureMap::default(),
            opaque_1024: TextureMap::default(),
            opaque_2048: TextureMap::default(),
            opaque_4096: TextureMap::default(),
            transparent_512: TextureMap::default(),
            transparent_1024: TextureMap::default(),
            transparent_2048: TextureMap::default(),
            transparent_4096: TextureMap::default(),
        }
    }

    pub fn add(
        &mut self,
        name: String,
        texture: Vec<u8>,
        texture_compression: graphics::gpu::TextureCompression,
    ) -> Result<(TextureArray, usize), GltfError> {
        let mut transcoder = basis_universal::transcoding::Transcoder::new();

        if !transcoder.validate_header(&texture) {
            return Err(GltfError::InvalidTexture { name });
        }

        let image_info = transcoder.image_info(&texture, 0).unwrap();
        println!("{:#?}", image_info);
        let width = image_info.m_width;
        let height = image_info.m_height;

        transcoder.prepare_transcoding(&texture).unwrap();

        let mut transcoded_texture = Vec::new();

        let target_texture_format = match texture_compression {
            graphics::gpu::TextureCompression::Astc => {
                basis_universal::transcoding::TranscoderTextureFormat::ASTC_4x4_RGBA
            }
            graphics::gpu::TextureCompression::Bc => {
                basis_universal::transcoding::TranscoderTextureFormat::BC7_RGBA
            }
            graphics::gpu::TextureCompression::None => {
                basis_universal::transcoding::TranscoderTextureFormat::RGBA32
            }
        };

        for level_index in 0..transcoder.image_level_count(&texture, 0) {
            let transcoded_level = transcoder
                .transcode_image_level(
                    &texture,
                    target_texture_format,
                    basis_universal::transcoding::TranscodeParameters {
                        image_index: 0,
                        level_index,
                        decode_flags: None,
                        output_row_pitch_in_blocks_or_pixels: None,
                        output_rows_in_pixels: None,
                    },
                )
                .or(Err(GltfError::TextureTranscodingError {
                    name: name.clone(),
                }))?;

            transcoded_texture.extend(transcoded_level);
        }

        transcoder.end_transcoding();

        let (texture_array, texture_index) = match (width, height) {
            (512, 512) => (
                TextureArray::Opaque512,
                self.opaque_512.add(name, transcoded_texture),
            ),
            (1024, 1024) => (
                TextureArray::Opaque1024,
                self.opaque_1024.add(name, transcoded_texture),
            ),
            (2048, 2048) => (
                TextureArray::Opaque2048,
                self.opaque_2048.add(name, transcoded_texture),
            ),
            (4096, 4096) => (
                TextureArray::Opaque4096,
                self.opaque_4096.add(name, transcoded_texture),
            ),
            _ => {
                return Err(GltfError::UnsupportedTextureSize {
                    name,
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
    pub map: HashMap<graphics::Material, usize>,
}

impl MaterialMap {
    pub fn add(&mut self, material: graphics::Material) -> usize {
        if let Some(&material_index) = self.map.get(&material) {
            return material_index;
        }

        let material_index = self.materials.len();

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
    TextureTranscodingError {
        name: String,
    },
    UnsupportedTextureSize {
        name: String,
        width: u32,
        height: u32,
    },
}

impl std::error::Error for GltfError {}

impl std::fmt::Display for GltfError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Self::InvalidTexture { name } => write!(f, "invalid texture with name \"{name}\""),
            Self::TextureTranscodingError { name } => {
                write!(f, "error transcoding texture with name \"{name}\"")
            }
            Self::UnsupportedTextureSize {
                name,
                width,
                height,
            } => {
                write!(
                    f,
                    "unsupported texture size {width}*{height} for \"{name}\"",
                )
            }
        }
    }
}
