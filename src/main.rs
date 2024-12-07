mod app;
mod asset;
mod ecs;
mod graphics;

fn main() {
    let mut asset_loader = asset::AssetLoader::new();
    asset_loader.load_gltf_model("assets/FlightHelmet.gltf");

    let asset::AssetLoader {
        mesh_map,
        texture_arrays,
        texture_dictionary,
        material_map,
        model_map,
    } = asset_loader;

    // Removing this makes wgpu fail silently.
    env_logger::init();

    app::App::run();
}
