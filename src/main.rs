mod app;
mod asset;
mod ecs;
mod graphics;
mod scene;

fn main() {
    scene::Scene::new();

    // Removing this makes wgpu fail silently.
    env_logger::init();

    app::App::run();
}
