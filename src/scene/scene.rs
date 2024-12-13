use crate::graphics;

pub trait Scene {
    fn update(&mut self);
    fn render(&mut self, gpu: &graphics::Gpu);
}
