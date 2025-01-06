use std::sync::Arc;

#[derive(bevy_ecs::prelude::Resource, Debug)]
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

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }
}

pub fn create_depth_buffer(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    msaa_sample_count: u32,
) -> wgpu::TextureView {
    let depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth_buffer"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: msaa_sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    depth_buffer.create_view(&wgpu::TextureViewDescriptor::default())
}

pub fn create_msaa_buffer(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    texture_format: wgpu::TextureFormat,
    msaa_sample_count: u32,
) -> wgpu::TextureView {
    let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("msaa_texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: msaa_sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: texture_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    msaa_texture.create_view(&wgpu::TextureViewDescriptor::default())
}
