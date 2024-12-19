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
    pub texture_dictionary: HashMap<String, (TextureArray, TextureId)>,
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

    pub fn load_gltf_model<P>(&mut self, path: P) -> Result<ModelId, AssetError>
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

                    let (base_color_texture_array, base_color_texture_id) =
                        if let gltf::image::Source::Uri { uri, .. } =
                            base_color_texture_information.texture().source().source()
                        {
                            // TODO: error message. error if invalid file path.
                            let texture_path = directory_path.join(uri).canonicalize().unwrap();
                            self.load_texture(&texture_path)?
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

                    let (normal_texture_array, normal_texture_id) =
                        if let gltf::image::Source::Uri { uri, .. } = gltf_material
                            .normal_texture()
                            .unwrap()
                            .texture()
                            .source()
                            .source()
                        {
                            // TODO: error message. error if invalid file path.
                            let texture_path = directory_path.join(uri).canonicalize().unwrap();
                            self.load_texture(&texture_path)?
                        } else {
                            // TODO: error message. Expected URI source.
                            panic!();
                        };

                    let metallic_roughness_texture_information =
                        pbr_metallic_roughness.metallic_roughness_texture().unwrap();

                    let (metallic_roughness_texture_array, metallic_roughness_texture_id) =
                        if let gltf::image::Source::Uri { uri, .. } =
                            metallic_roughness_texture_information
                                .texture()
                                .source()
                                .source()
                        {
                            // TODO: error message. error if invalid file path.
                            let texture_path = directory_path.join(uri).canonicalize().unwrap();
                            self.load_texture(&texture_path)?
                        } else {
                            // TODO: error message. Expected URI source.
                            panic!();
                        };

                    let material = graphics::Material::new(
                        base_color_texture_array.id(),
                        base_color_texture_id,
                        normal_texture_array.id(),
                        normal_texture_id,
                        metallic_roughness_texture_array.id(),
                        metallic_roughness_texture_id,
                    );

                    let material_index = self.material_map.add(material);

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
                            [tangent[0], tangent[1], tangent[2]],
                            bitangent.into(),
                        ));
                    }

                    let indices = reader
                        .read_indices()
                        .unwrap() // TODO: error message.
                        .into_u32()
                        .collect::<Vec<_>>();

                    let gltf_bounding_box = primitive.bounding_box();
                    let bounding_box =
                        graphics::BoundingBox::new(gltf_bounding_box.min, gltf_bounding_box.max);

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

    pub fn load_texture<P>(
        &mut self,
        texture_path: P,
    ) -> Result<(TextureArray, TextureId), AssetError>
    where
        P: AsRef<std::path::Path>,
    {
        let texture_path_str = texture_path.as_ref().to_str().unwrap();

        let (texture_array, texture_id) = if let Some((texture_array, texture_id)) =
            self.texture_dictionary.get(texture_path_str)
        {
            (*texture_array, *texture_id)
        } else {
            let texture_data = std::fs::read(texture_path_str).unwrap();
            let texture_reader = ktx2::Reader::new(&texture_data).unwrap();
            let (texture_array, texture_id) = self
                .texture_arrays
                .add(texture_path_str.to_string(), texture_reader)?;
            self.texture_dictionary
                .insert(texture_path_str.to_string(), (texture_array, texture_id));
            (texture_array, texture_id)
        };

        Ok((texture_array, texture_id))
    }

    pub fn load_cubemap<P>(
        &mut self,
        path_positive_x: P,
        path_negative_x: P,
        path_positive_y: P,
        path_negative_y: P,
        path_positive_z: P,
        path_negative_z: P,
    ) -> Result<Cubemap, AssetError>
    where
        P: AsRef<std::path::Path>,
    {
        let (texture_array_positive_x, positive_x) = self.load_texture(&path_positive_x)?;
        let texture_array = texture_array_positive_x;

        const VALID_CUBEMAP_TEXTURE_ARRAYS: [TextureArray; 1] =
            [TextureArray::NoMipRgbBc6hSFloat1024];
        if !VALID_CUBEMAP_TEXTURE_ARRAYS.contains(&texture_array) {
            return Err(AssetError::InvalidCubemapTexture {
                name: path_positive_x.as_ref().to_str().unwrap().to_string(),
            });
        }

        let (texture_array_negative_x, negative_x) = self.load_texture(&path_negative_x)?;
        if texture_array_negative_x != texture_array {
            return Err(AssetError::NonMatchingCubemapTexture {
                name: path_negative_x.as_ref().to_str().unwrap().to_string(),
            });
        }

        let (texture_array_positive_y, positive_y) = self.load_texture(&path_positive_y)?;
        if texture_array_positive_y != texture_array {
            return Err(AssetError::NonMatchingCubemapTexture {
                name: path_positive_y.as_ref().to_str().unwrap().to_string(),
            });
        }

        let (texture_array_negative_y, negative_y) = self.load_texture(&path_negative_y)?;
        if texture_array_negative_y != texture_array {
            return Err(AssetError::NonMatchingCubemapTexture {
                name: path_negative_y.as_ref().to_str().unwrap().to_string(),
            });
        }

        let (texture_array_positive_z, positive_z) = self.load_texture(&path_positive_z)?;
        if texture_array_positive_z != texture_array {
            return Err(AssetError::NonMatchingCubemapTexture {
                name: path_positive_z.as_ref().to_str().unwrap().to_string(),
            });
        }

        let (texture_array_negative_z, negative_z) = self.load_texture(&path_negative_z)?;
        if texture_array_negative_z != texture_array {
            return Err(AssetError::NonMatchingCubemapTexture {
                name: path_negative_z.as_ref().to_str().unwrap().to_string(),
            });
        }

        Ok(Cubemap {
            texture_array,
            positive_x,
            negative_x,
            positive_y,
            negative_y,
            positive_z,
            negative_z,
        })
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

pub struct TextureMap {
    pub map: HashMap<String, u32>,
    pub mip_levels: Vec<(usize, usize)>, // (data_offset, data_length)
    pub dimension: u32,
    pub mip_level_count: u32,
    pub data: Vec<u8>,
    pub format: wgpu::TextureFormat,
}

impl TextureMap {
    pub fn new(dimension: u32, format: wgpu::TextureFormat, mip_level_count: u32) -> Self {
        Self {
            map: HashMap::new(),
            mip_levels: vec![],
            dimension,
            mip_level_count,
            data: vec![],
            format,
        }
    }

    pub fn count(&self) -> usize {
        self.mip_levels.len() / self.mip_level_count as usize
    }

    pub fn add(&mut self, name: String, texture: ktx2::Reader<&Vec<u8>>) -> TextureId {
        if let Some(&texture_index) = self.map.get(&name) {
            return texture_index;
        }

        for mip_level in texture.levels() {
            let offset = self.data.len();
            self.data.extend(mip_level);
            self.mip_levels.push((offset, mip_level.len()));
        }

        let texture_index = self.map.len() as u32;
        self.map.insert(name, texture_index);

        texture_index
    }

    pub fn get(&self, layer_index: u32, mip_level_index: u32) -> Result<&[u8], AssetError> {
        if mip_level_index >= self.mip_level_count {
            return Err(AssetError::InvalidMipLevel {
                mip_level: mip_level_index,
            });
        }

        let texture_information_index =
            (layer_index * self.mip_level_count + mip_level_index) as usize;
        if texture_information_index >= self.mip_levels.len() {
            return Err(AssetError::TextureLayerOutOfBounds { layer_index });
        }
        let (data_offset, data_length) = self.mip_levels[texture_information_index];

        Ok(&self.data[data_offset..(data_offset + data_length)])
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
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
    NoMipRgbBc6hSFloat1024,
}

impl TextureArray {
    pub const fn id(&self) -> TextureArrayId {
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
            Self::NoMipRgbBc6hSFloat1024 => 12,
        }
    }

    pub const fn size(&self) -> (usize, usize) {
        match self {
            Self::RgBc5Unorm512 | Self::RgbBc7Unorm512 | Self::RgbaBc7Srgb512 => (512, 512),
            Self::RgBc5Unorm1024
            | Self::RgbBc7Unorm1024
            | Self::RgbaBc7Srgb1024
            | Self::NoMipRgbBc6hSFloat1024 => (1024, 1024),
            Self::RgBc5Unorm2048 | Self::RgbBc7Unorm2048 | Self::RgbaBc7Srgb2048 => (2048, 2048),
            Self::RgBc5Unorm4096 | Self::RgbBc7Unorm4096 | Self::RgbaBc7Srgb4096 => (4096, 4096),
        }
    }

    pub const fn mip_level_count(&self) -> u32 {
        match self {
            Self::RgBc5Unorm512 | Self::RgbBc7Unorm512 | Self::RgbaBc7Srgb512 => 10,
            Self::RgBc5Unorm1024 | Self::RgbBc7Unorm1024 | Self::RgbaBc7Srgb1024 => 11,
            Self::RgBc5Unorm2048 | Self::RgbBc7Unorm2048 | Self::RgbaBc7Srgb2048 => 12,
            Self::RgBc5Unorm4096 | Self::RgbBc7Unorm4096 | Self::RgbaBc7Srgb4096 => 13,
            Self::NoMipRgbBc6hSFloat1024 => 1,
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
    pub no_mip_rgb_bc6h_sfloat_1024: TextureMap,
}

impl TextureArrays {
    pub fn new() -> Self {
        Self {
            rg_bc5_unorm_512: TextureMap::new(
                512,
                wgpu::TextureFormat::Bc5RgUnorm,
                TextureArray::RgBc5Unorm512.mip_level_count(),
            ),
            rg_bc5_unorm_1024: TextureMap::new(
                1024,
                wgpu::TextureFormat::Bc5RgUnorm,
                TextureArray::RgBc5Unorm1024.mip_level_count(),
            ),
            rg_bc5_unorm_2048: TextureMap::new(
                2048,
                wgpu::TextureFormat::Bc5RgUnorm,
                TextureArray::RgBc5Unorm2048.mip_level_count(),
            ),
            rg_bc5_unorm_4096: TextureMap::new(
                4096,
                wgpu::TextureFormat::Bc5RgUnorm,
                TextureArray::RgBc5Unorm4096.mip_level_count(),
            ),
            rgb_bc7_unorm_512: TextureMap::new(
                512,
                wgpu::TextureFormat::Bc7RgbaUnorm,
                TextureArray::RgbBc7Unorm512.mip_level_count(),
            ),
            rgb_bc7_unorm_1024: TextureMap::new(
                1024,
                wgpu::TextureFormat::Bc7RgbaUnorm,
                TextureArray::RgbBc7Unorm1024.mip_level_count(),
            ),
            rgb_bc7_unorm_2048: TextureMap::new(
                2048,
                wgpu::TextureFormat::Bc7RgbaUnorm,
                TextureArray::RgbBc7Unorm2048.mip_level_count(),
            ),
            rgb_bc7_unorm_4096: TextureMap::new(
                4096,
                wgpu::TextureFormat::Bc7RgbaUnorm,
                TextureArray::RgbBc7Unorm4096.mip_level_count(),
            ),
            rgba_bc7_srgb_512: TextureMap::new(
                512,
                wgpu::TextureFormat::Bc7RgbaUnormSrgb,
                TextureArray::RgbaBc7Srgb512.mip_level_count(),
            ),
            rgba_bc7_srgb_1024: TextureMap::new(
                1024,
                wgpu::TextureFormat::Bc7RgbaUnormSrgb,
                TextureArray::RgbaBc7Srgb1024.mip_level_count(),
            ),
            rgba_bc7_srgb_2048: TextureMap::new(
                2048,
                wgpu::TextureFormat::Bc7RgbaUnormSrgb,
                TextureArray::RgbaBc7Srgb2048.mip_level_count(),
            ),
            rgba_bc7_srgb_4096: TextureMap::new(
                4096,
                wgpu::TextureFormat::Bc7RgbaUnormSrgb,
                TextureArray::RgbaBc7Srgb4096.mip_level_count(),
            ),
            no_mip_rgb_bc6h_sfloat_1024: TextureMap::new(
                1024,
                wgpu::TextureFormat::Bc6hRgbFloat,
                TextureArray::NoMipRgbBc6hSFloat1024.mip_level_count(),
            ),
        }
    }

    pub fn add(
        &mut self,
        name: String,
        texture: ktx2::Reader<&Vec<u8>>,
    ) -> Result<(TextureArray, u32), AssetError> {
        let ktx2::Header {
            format,
            pixel_width: width,
            pixel_height: height,
            level_count: mip_level_count,
            ..
        } = texture.header();

        let (texture_array, texture_index) = match (format, width, height, mip_level_count) {
            (Some(ktx2::Format::BC5_UNORM_BLOCK), 512, 512, 10) => (
                TextureArray::RgBc5Unorm512,
                self.rg_bc5_unorm_512.add(name, texture),
            ),
            (Some(ktx2::Format::BC5_UNORM_BLOCK), 1024, 1024, 11) => (
                TextureArray::RgBc5Unorm1024,
                self.rg_bc5_unorm_1024.add(name, texture),
            ),
            (Some(ktx2::Format::BC5_UNORM_BLOCK), 2048, 2048, 12) => (
                TextureArray::RgBc5Unorm2048,
                self.rg_bc5_unorm_2048.add(name, texture),
            ),
            (Some(ktx2::Format::BC5_UNORM_BLOCK), 4096, 4096, 13) => (
                TextureArray::RgBc5Unorm4096,
                self.rg_bc5_unorm_4096.add(name, texture),
            ),
            (Some(ktx2::Format::BC7_UNORM_BLOCK), 512, 512, 10) => (
                TextureArray::RgbBc7Unorm512,
                self.rgb_bc7_unorm_512.add(name, texture),
            ),
            (Some(ktx2::Format::BC7_UNORM_BLOCK), 1024, 1024, 11) => (
                TextureArray::RgbBc7Unorm1024,
                self.rgb_bc7_unorm_1024.add(name, texture),
            ),
            (Some(ktx2::Format::BC7_UNORM_BLOCK), 2048, 2048, 12) => (
                TextureArray::RgbBc7Unorm2048,
                self.rgb_bc7_unorm_2048.add(name, texture),
            ),
            (Some(ktx2::Format::BC7_UNORM_BLOCK), 4096, 4096, 13) => (
                TextureArray::RgbBc7Unorm4096,
                self.rgb_bc7_unorm_4096.add(name, texture),
            ),
            (Some(ktx2::Format::BC7_SRGB_BLOCK), 512, 512, 10) => (
                TextureArray::RgbaBc7Srgb512,
                self.rgba_bc7_srgb_512.add(name, texture),
            ),
            (Some(ktx2::Format::BC7_SRGB_BLOCK), 1024, 1024, 11) => (
                TextureArray::RgbaBc7Srgb1024,
                self.rgba_bc7_srgb_1024.add(name, texture),
            ),
            (Some(ktx2::Format::BC7_SRGB_BLOCK), 2048, 2048, 12) => (
                TextureArray::RgbaBc7Srgb2048,
                self.rgba_bc7_srgb_2048.add(name, texture),
            ),
            (Some(ktx2::Format::BC7_SRGB_BLOCK), 4096, 4096, 13) => (
                TextureArray::RgbaBc7Srgb4096,
                self.rgba_bc7_srgb_4096.add(name, texture),
            ),
            (Some(ktx2::Format::BC6H_SFLOAT_BLOCK), 1024, 1024, 1) => (
                TextureArray::NoMipRgbBc6hSFloat1024,
                self.no_mip_rgb_bc6h_sfloat_1024.add(name, texture),
            ),
            _ => {
                return Err(AssetError::UnsupportedTextureFormat {
                    name,
                    format,
                    width,
                    height,
                    mip_level_count,
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

pub struct Cubemap {
    pub texture_array: TextureArray,
    pub positive_x: TextureId,
    pub negative_x: TextureId,
    pub positive_y: TextureId,
    pub negative_y: TextureId,
    pub positive_z: TextureId,
    pub negative_z: TextureId,
}

#[derive(Eq, PartialEq, Debug)]
pub enum AssetError {
    InvalidTexture {
        name: String,
    },
    UnsupportedTextureFormat {
        name: String,
        format: Option<ktx2::Format>,
        width: u32,
        height: u32,
        mip_level_count: u32,
    },
    InvalidCubemapTexture {
        name: String,
    },
    NonMatchingCubemapTexture {
        name: String,
    },
    InvalidMipLevel {
        mip_level: u32,
    },
    TextureLayerOutOfBounds {
        layer_index: u32,
    },
}

impl std::error::Error for AssetError {}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Self::InvalidTexture { name } => write!(f, "invalid texture with name \"{name}\""),
            Self::UnsupportedTextureFormat {
                name,
                format,
                width,
                height,
                mip_level_count,
            } => {
                write!(
                    f,
                    "unsupported texture format {:?} with size {width}*{height} and mip level count {mip_level_count} for \"{name}\"",
                    format
                )
            }
            Self::InvalidCubemapTexture { name } => {
                write!(f, "invalid cubemap texture with name \"{name}\"")
            }
            Self::NonMatchingCubemapTexture { name } => {
                write!(
                    f,
                    "format of texture with name \"{name}\" differs from other cubemap faces"
                )
            }
            Self::InvalidMipLevel { mip_level } => {
                write!(f, "invalid mip level \"{mip_level}\"")
            }
            Self::TextureLayerOutOfBounds { layer_index } => {
                write!(f, "texture layer \"{layer_index}\" out of bounds")
            }
        }
    }
}
