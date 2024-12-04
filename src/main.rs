mod app;
mod graphics;

fn main() {
    let mut mesh_map = graphics::mesh::MeshMap::default();

    let (gltf, buffers, _) = gltf::import("assets/FlightHelmet.gltf").unwrap();
    for mesh in gltf.meshes() {
        for (i, primitive) in mesh.primitives().enumerate() {
            let name = format!("{}/{}", mesh.name().unwrap(), i);

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let vertex_positions = reader.read_positions().unwrap(); // TODO: error message.
            let vertex_normals = reader.read_normals().unwrap(); // TODO: error message.
            let vertex_tangents = reader.read_tangents().unwrap(); // TODO: error message.

            let mut vertices = Vec::with_capacity(vertex_positions.len());
            for (position, normal, tangent) in
                itertools::izip!(vertex_positions, vertex_normals, vertex_tangents)
            {
                vertices.push(graphics::mesh::Vertex::new(position, normal, tangent));
            }

            let indices = reader
                .read_indices()
                .unwrap() // TODO: error message.
                .into_u32()
                .collect::<Vec<_>>();

            let mesh_index = mesh_map.push(name, vertices, indices);
        }
    }

    // Removing this makes wgpu fail silently.
    env_logger::init();

    app::App::run();
}
