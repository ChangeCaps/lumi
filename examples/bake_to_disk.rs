use std::fs;

use lumi::util::DeviceExt;
use lumi_bake::{BakeInstance, BakedEnvironment};

fn main() {
    let instance = BakeInstance::new().unwrap();
    let image = image::open("env.hdr").unwrap();
    let data = image.into_rgba16();
    let bytes: &[u8] = bytemuck::cast_slice(data.as_raw());

    let texture = instance.device().create_texture_with_data(
        instance.queue(),
        &wgpu::TextureDescriptor {
            label: Some("Lumi Bake Texture"),
            size: wgpu::Extent3d {
                width: data.width(),
                height: data.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        },
        bytes,
    );

    let baked_env =
        BakedEnvironment::from_eq(instance.device(), instance.queue(), &texture, data.height());
    let data = baked_env.to_data(instance.device(), instance.queue());

    let file = fs::File::create("env.bake").unwrap();
    data.save(file).unwrap();
}
