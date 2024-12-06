use std::collections::HashMap;
use std::hash::Hash;

#[derive(Default, Debug)]
pub struct TextureMap {
    pub textures: Vec<u8>,
    pub map: HashMap<String, usize>,
}

impl TextureMap {
    pub fn add(&mut self, name: String, texture: ktx2::Reader<Vec<u8>>) -> usize {
        if let Some(&texture_index) = self.map.get(&name) {
            return texture_index;
        }

        let texture_index = self.map.len();

        self.textures.extend(texture.data());
        self.map.insert(name, texture_index);

        texture_index
    }

    pub fn get(&self, name: &str) -> Option<usize> {
        self.map.get(name).copied()
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum TextureFormat {
    Opaque512,
    Opaque1024,
    Opaque2048,
    Opaque4096,
    Transparent512,
    Transparent1024,
    Transparent2048,
    Transparent4096,
}

impl TextureFormat {
    pub fn id(&self) -> usize {
        match self {
            Self::Opaque512 => 0,
            Self::Opaque1024 => 1,
            Self::Opaque2048 => 2,
            Self::Opaque4096 => 3,
            Self::Transparent512 => 4,
            Self::Transparent1024 => 5,
            Self::Transparent2048 => 6,
            Self::Transparent4096 => 7,
        }
    }
}

pub struct TextureArrays {
    map: HashMap<TextureFormat, TextureMap>,
}

impl TextureArrays {
    pub fn new() -> Self {
        Self {
            map: HashMap::from([
                (TextureFormat::Opaque512, TextureMap::default()),
                (TextureFormat::Opaque1024, TextureMap::default()),
                (TextureFormat::Opaque2048, TextureMap::default()),
                (TextureFormat::Opaque4096, TextureMap::default()),
                (TextureFormat::Transparent512, TextureMap::default()),
                (TextureFormat::Transparent1024, TextureMap::default()),
                (TextureFormat::Transparent2048, TextureMap::default()),
                (TextureFormat::Transparent4096, TextureMap::default()),
            ]),
        }
    }

    pub fn add(
        &mut self,
        name: String,
        texture: ktx2::Reader<Vec<u8>>,
    ) -> Result<(TextureFormat, usize), TextureError> {
        let ktx2::Header {
            format,
            pixel_width,
            pixel_height,
            ..
        } = texture.header();

        let texture_format = match (format, pixel_width, pixel_height) {
            (Some(ktx2::Format::R8G8B8_SRGB), 512, 512) => TextureFormat::Opaque512,
            (Some(ktx2::Format::R8G8B8_SRGB), 1024, 1024) => TextureFormat::Opaque1024,
            (Some(ktx2::Format::R8G8B8_SRGB), 2048, 2048) => TextureFormat::Opaque2048,
            (Some(ktx2::Format::R8G8B8_SRGB), 4096, 4096) => TextureFormat::Opaque4096,
            (Some(ktx2::Format::R8G8B8A8_SRGB), 512, 512) => TextureFormat::Transparent512,
            (Some(ktx2::Format::R8G8B8A8_SRGB), 1024, 1024) => TextureFormat::Transparent1024,
            (Some(ktx2::Format::R8G8B8A8_SRGB), 2048, 2048) => TextureFormat::Transparent2048,
            (Some(ktx2::Format::R8G8B8A8_SRGB), 4096, 4096) => TextureFormat::Transparent4096,
            _ => {
                return Err(TextureError::UnsupportedTextureFormat {
                    format,
                    pixel_width,
                    pixel_height,
                })
            }
        };

        let texture_map = self.map.get_mut(&texture_format).unwrap();
        let texture_index = texture_map.add(name, texture);

        Ok((texture_format, texture_index))
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum TextureError {
    UnsupportedTextureFormat {
        format: Option<ktx2::Format>,
        pixel_width: u32,
        pixel_height: u32,
    },
}

impl std::error::Error for TextureError {}

impl std::fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Self::UnsupportedTextureFormat {
                format,
                pixel_width,
                pixel_height,
            } => {
                write!(
                    f,
                    "unsupported texture format {:?} of size {pixel_width}*{pixel_height}",
                    format
                )
            }
        }
    }
}
