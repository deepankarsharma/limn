use std::path::Path;

use gfx_device_gl::Factory;
use gfx_graphics::{TextureSettings, Flip};
use resources::{Map, ImageId};
use backend::gfx::G2dTexture;

pub type Texture = G2dTexture<'static>;

impl Map<ImageId, Texture> {
    pub fn insert_from_file<P>(&mut self, factory: &mut Factory, path: P) -> ImageId
        where P: AsRef<Path>
    {
        let settings = TextureSettings::new();
        let image = Texture::from_path(factory, &path, Flip::None, &settings).unwrap();
        self.insert(image)
    }
}
