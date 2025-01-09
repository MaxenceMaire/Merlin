use bevy_ecs::component::Component;
use std::ops::{Deref, DerefMut};

#[derive(Component, Copy, Clone, Debug)]
pub struct GlobalTransform(pub glam::Affine3A);

impl Default for GlobalTransform {
    fn default() -> Self {
        Self(glam::Affine3A::IDENTITY)
    }
}

impl Deref for GlobalTransform {
    type Target = glam::Affine3A;

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
