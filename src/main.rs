mod app;
mod graphics;

fn main() {
    // Removing this makes wgpu fail silently.
    env_logger::init();

    app::App::run();
}
