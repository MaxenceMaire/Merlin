use crate::graphics;
use std::collections::{HashMap, VecDeque};

type TextureArrayId = usize;
type TextureId = usize;
type MeshId = usize;
type MaterialId = usize;
type ModelId = usize;

pub struct AssetLoader {
    pub mesh_map: graphics::mesh::MeshMap,
    pub texture_arrays: graphics::texture::TextureArrays,
    pub texture_dictionary: HashMap<String, (TextureArrayId, TextureId)>,
    pub material_map: graphics::material::MaterialMap,
    pub model_map: ModelMap,
}

impl AssetLoader {
    pub fn new() -> Self {
        Self {
            mesh_map: graphics::mesh::MeshMap::default(),
            texture_arrays: graphics::texture::TextureArrays::new(),
            texture_dictionary: HashMap::new(),
            material_map: graphics::material::MaterialMap::default(),
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

        let mut load_texture = |texture_path: &std::path::Path| -> Result<
            (TextureArrayId, TextureId),
            graphics::texture::TextureError,
        > {
            let texture_path_str = texture_path.to_str().unwrap();

            let (texture_array_id, texture_id) = if let Some((texture_array_id, texture_id)) =
                self.texture_dictionary.get(texture_path_str)
            {
                (*texture_array_id, *texture_id)
            } else {
                let texture_data = std::fs::read(texture_path_str).unwrap();
                let ktx2_reader = ktx2::Reader::new(texture_data).unwrap();
                let (format, texture_id) = self
                    .texture_arrays
                    .add(texture_path_str.to_string(), ktx2_reader)?;
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

                    let material = graphics::material::Material::new(
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
                        vertices.push(graphics::mesh::Vertex::new(
                            position, tex_coord, normal, tangent,
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

                    object_group.objects.push((mesh_index, material_index));
                }

                node.object_group = Some(object_group);
            }

            for child in gltf_node.children() {
                stack.push_back(child);
                node.children.push(nodes.len());
            }

            nodes.push(node);
        }

        let model = Model { nodes };

        println!("{:#?}", self.texture_dictionary);
        println!("{:#?} {:#?}", self.mesh_map.meshes, self.mesh_map.map);
        println!("{:#?}", self.material_map);
        println!("{:#?}", model);

        let model_index = self
            .model_map
            .add(path.to_str().unwrap().to_string(), model);

        Ok(model_index)
    }
}

#[derive(Debug)]
pub struct Model {
    nodes: Vec<Node>,
}

#[derive(Debug)]
pub struct Node {
    name: Option<String>,
    object_group: Option<ObjectGroup>,
    children: Vec<usize>,
}

type Object = (MeshId, MaterialId);

#[derive(Debug)]
pub struct ObjectGroup {
    objects: Vec<Object>,
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
}

#[derive(Eq, PartialEq, Debug)]
pub enum GltfError {
    TextureError(graphics::texture::TextureError),
}

impl std::error::Error for GltfError {}

impl std::fmt::Display for GltfError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Self::TextureError(texture_error) => {
                write!(f, "texture error: {texture_error}")
            }
        }
    }
}

impl From<graphics::texture::TextureError> for GltfError {
    fn from(error: graphics::texture::TextureError) -> Self {
        Self::TextureError(error)
    }
}
