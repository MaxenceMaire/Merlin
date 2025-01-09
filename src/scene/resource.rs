use crate::graphics;

use std::ops::{Deref, DerefMut};

#[derive(bevy_ecs::system::Resource)]
pub struct Timestamp(pub std::time::Instant);

impl Deref for Timestamp {
    type Target = std::time::Instant;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Timestamp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::system::Resource)]
pub struct DeltaTime(pub std::time::Duration);

impl Deref for DeltaTime {
    type Target = std::time::Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DeltaTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::system::Resource)]
pub struct Meshes(pub Vec<graphics::Mesh>);

impl Deref for Meshes {
    type Target = Vec<graphics::Mesh>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Meshes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::system::Resource)]
pub struct VertexBuffer(pub wgpu::Buffer);

impl Deref for VertexBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VertexBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::system::Resource)]
pub struct IndexBuffer(pub wgpu::Buffer);

impl Deref for IndexBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for IndexBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::system::Resource)]
pub struct BoundingBoxesBuffer(pub wgpu::Buffer);

impl Deref for BoundingBoxesBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BoundingBoxesBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::system::Resource)]
pub struct DepthBuffer(pub wgpu::TextureView);

impl Deref for DepthBuffer {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DepthBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::system::Resource)]
pub struct MsaaBuffer(pub wgpu::TextureView);

impl Deref for MsaaBuffer {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MsaaBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::system::Resource)]
pub struct ComputePipelineFrustumCulling(pub graphics::pipeline::compute::FrustumCulling);

impl Deref for ComputePipelineFrustumCulling {
    type Target = graphics::pipeline::compute::FrustumCulling;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ComputePipelineFrustumCulling {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::system::Resource)]
pub struct RenderPipelinePbr(pub graphics::pipeline::render::Pbr);

impl Deref for RenderPipelinePbr {
    type Target = graphics::pipeline::render::Pbr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RenderPipelinePbr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::system::Resource)]
pub struct RenderPipelineSkybox(pub graphics::pipeline::render::Skybox);

impl Deref for RenderPipelineSkybox {
    type Target = graphics::pipeline::render::Skybox;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RenderPipelineSkybox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::system::Resource)]
pub struct BindGroupBindless(pub wgpu::BindGroup);

impl Deref for BindGroupBindless {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BindGroupBindless {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::system::Resource)]
pub struct BindGroupSkybox(pub wgpu::BindGroup);

impl Deref for BindGroupSkybox {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BindGroupSkybox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
