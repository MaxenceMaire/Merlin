mod app;
mod asset;
mod ecs;
mod graphics;
mod physics;
mod scene;

fn main() {
    // Removing this makes wgpu fail silently.
    env_logger::init();

    app::App::run();
}
