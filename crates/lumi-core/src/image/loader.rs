use lumi_assets::{AssetLoader, LoadContext};
use lumi_util::{async_trait, bytemuck};
use wgpu::TextureFormat;

use crate::{Image, ImageData};

pub struct ImageLoader;

#[async_trait]
impl AssetLoader for ImageLoader {
    async fn load(&self, ctx: &LoadContext<'_>) -> Result<(), ()> {
        let image = image::load_from_memory(ctx.bytes).unwrap();
        let width = image.width();
        let height = image.height();

        match ctx.extension {
            "hdr" => {
                let data = image
                    .into_rgba16()
                    .into_raw()
                    .into_iter()
                    .map(bytemuck::cast::<_, [u8; 2]>)
                    .flatten()
                    .collect();
                let image = Image::new(ImageData::with_format(
                    width,
                    height,
                    data,
                    TextureFormat::Rgba16Uint,
                ));
                ctx.handle.set(image).unwrap();
            }
            _ => {
                let data = image.into_rgba8().into_raw();
                let image = Image::new(ImageData::new(width, height, data));
                ctx.handle.set(image).unwrap();
            }
        }

        Ok(())
    }

    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "hdr"]
    }
}
