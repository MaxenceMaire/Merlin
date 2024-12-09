use crate::graphics;

pub trait Scene {
    fn update(&mut self);
    fn render(&self, gpu: &graphics::Gpu);
}
