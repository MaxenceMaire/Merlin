use bevy_ecs::component::Component;

#[repr(C)]
#[derive(Component, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct GlobalTransform([f32; 12]);

#[derive(Component)]
pub struct Transform {
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}
