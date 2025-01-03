use std::sync::Arc;

pub struct Gpu<'a> {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'a>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl<'a> Gpu<'a> {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let window_size = window.inner_size();

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width.max(1),
            height: window_size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let mut required_features = wgpu::Features::empty();

        if !adapter
            .features()
            .contains(wgpu::Features::TEXTURE_COMPRESSION_BC)
        {
            panic!("BCn texture compression not supported by GPU");
        } else {
            required_features.set(wgpu::Features::TEXTURE_COMPRESSION_BC, true);
        }

        if !adapter
            .features()
            .contains(wgpu::Features::INDIRECT_FIRST_INSTANCE)
        {
            panic!("indirect first instance feature not supported by GPU");
        } else {
            required_features.set(wgpu::Features::INDIRECT_FIRST_INSTANCE, true);
        }

        if !adapter
            .features()
            .contains(wgpu::Features::MULTI_DRAW_INDIRECT)
        {
            panic!("indirect first instance feature not supported by GPU");
        } else {
            required_features.set(wgpu::Features::MULTI_DRAW_INDIRECT, true);
        }

        if !adapter.get_downlevel_capabilities().flags.contains(wgpu::DownlevelFlags::VERTEX_AND_INSTANCE_INDEX_RESPECTS_RESPECTIVE_FIRST_VALUE_IN_INDIRECT_DRAW) {
            panic!("VERTEX_AND_INSTANCE_INDEX_RESPECTS_RESPECTIVE_FIRST_VALUE_IN_INDIRECT_DRAW not supported by GPU");
        }

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features,
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        surface.configure(&device, &config);

        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }
}
