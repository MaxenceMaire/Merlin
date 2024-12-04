pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0, 0.0).into(),
            target: (0.0, 1.0, 0.0).into(),
            up: cgmath::Vector3::unit_z(),
            aspect: 16.0 / 9.0, // TODO: read from config.
            fov: 90.0,
            near: 0.1,
            far: 100.0,
        }
    }
}
