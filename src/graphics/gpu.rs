use std::sync::Arc;

pub struct Gpu<'a> {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl<'a> Gpu<'a> {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    label: None,
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        Self {
            instance,
            surface,
            device,
            queue,
        }
    }
}
