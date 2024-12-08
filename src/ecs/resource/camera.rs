use bevy_ecs::system::Resource;

#[derive(Resource, Debug)]
pub struct Camera {
    pub position: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: (0.0, 5.0, 0.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: glam::Vec3::Z,
            fov: 90.0,
            near: 0.1,
            far: 100.0,
        }
    }
}
