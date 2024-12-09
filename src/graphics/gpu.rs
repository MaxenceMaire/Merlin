use std::sync::Arc;

pub struct Gpu<'a> {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'a>,
    pub adapter: wgpu::Adapter,
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

        let mut required_features = wgpu::Features::empty();

        if !adapter.features().contains(
            wgpu::Features::TEXTURE_COMPRESSION_ASTC | wgpu::Features::TEXTURE_COMPRESSION_ASTC_HDR,
        ) {
            panic!("ASTC texture compression not supported by GPU");
        } else {
            required_features.set(wgpu::Features::TEXTURE_COMPRESSION_ASTC, true);
            required_features.set(wgpu::Features::TEXTURE_COMPRESSION_ASTC_HDR, true);
        }

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features,
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
            adapter,
            device,
            queue,
        }
    }
}
