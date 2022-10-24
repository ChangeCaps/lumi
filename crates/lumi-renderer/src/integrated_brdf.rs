use lumi_bind::Bind;
use lumi_core::{Image, ImageData, TextureFormat};

#[derive(Clone, Debug, Bind)]
pub struct IntegratedBrdf {
    #[texture(name = "integrated_brdf")]
    #[sampler(name = "integrated_brdf_sampler")]
    pub image: Image,
}

impl Default for IntegratedBrdf {
    #[inline]
    fn default() -> Self {
        let data = ImageData::with_format(
            256,
            256,
            include_bytes!("integrated_brdf").to_vec(),
            TextureFormat::Rgba8Unorm,
        );

        Self {
            image: Image::new(data),
        }
    }
}
