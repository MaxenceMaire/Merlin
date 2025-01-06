use crate::graphics;
use crate::scene;
use scene::Scene;

use std::sync::Arc;

pub enum AppState {
    Initialized(App),
    Uninitialized,
}

impl AppState {
    pub fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match self {
            Self::Initialized(_) => panic!("app already initialized"),
            Self::Uninitialized => *self = Self::Initialized(App::new(event_loop)),
        }
    }
}

impl winit::application::ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match self {
            Self::Initialized(_) => todo!(),
            Self::Uninitialized => self.init(event_loop),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let app = match self {
            AppState::Initialized(app) => app,
            AppState::Uninitialized => unreachable!("uninitialized app"),
        };

        if window_id != app.window.id() {
            return;
        }

        match event {
            winit::event::WindowEvent::CloseRequested
            | winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: winit::event::ElementState::Pressed,
                        physical_key:
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(physical_size) => app.resize(physical_size),
            winit::event::WindowEvent::RedrawRequested => {
                app.scene.update();
                app.scene.extract();
                app.window.request_redraw();
            }
            _ => {}
        }
    }
}

pub struct App {
    window: Arc<winit::window::Window>, // TODO: remove Arc?
    scene: Scene,
}

impl App {
    pub fn run() {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut app_state = AppState::Uninitialized;
        event_loop.run_app(&mut app_state).unwrap();
    }

    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_title(String::from("Merlin"))
                        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None))),
                )
                .unwrap(),
        );

        let gpu = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(graphics::Gpu::new(window.clone()));

        let scene = Scene::setup(gpu);

        Self { window, scene }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // TODO: implement resize.
    }
}
