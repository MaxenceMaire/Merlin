use std::ops::{Deref, DerefMut};

#[derive(Default, Debug)]
pub struct TextureArray(Vec<u8>);

impl Deref for TextureArray {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TextureArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
