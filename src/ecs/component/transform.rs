use bevy_ecs::component::Component;
use std::ops::{Deref, DerefMut};

#[repr(C)]
#[derive(Component, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default, Debug)]
pub struct GlobalTransform(pub [f32; 12]);

impl Deref for GlobalTransform {
    type Target = [f32; 12];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GlobalTransform {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component, Default, Debug)]
pub struct Transform {
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}
