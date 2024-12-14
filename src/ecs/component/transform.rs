use bevy_ecs::component::Component;
use std::ops::{Deref, DerefMut};

#[repr(C)]
#[derive(Component, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct GlobalTransform(pub [f32; 12]);

impl Default for GlobalTransform {
    fn default() -> Self {
        Self(glam::Affine3A::IDENTITY.to_cols_array())
    }
}

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

#[derive(Component, Debug)]
pub struct Transform {
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}
