mod app;
mod asset;
mod graphics;

fn main() {
    let mut asset_loader = asset::AssetLoader::new();
    asset_loader.load_gltf_model("assets/FlightHelmet.gltf");

    // Removing this makes wgpu fail silently.
    env_logger::init();

    app::App::run();
}
