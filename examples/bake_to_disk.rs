use lumi_bake::{BakeInstance, BakedEnvironment};

fn main() {
    env_logger::builder()
        .filter_module("lumi_bake", log::LevelFilter::Trace)
        .init();

    let instance = BakeInstance::new().unwrap();
    let baked_env = BakedEnvironment::open_from_eq(
        instance.device(),
        instance.queue(),
        "env.hdr",
        384,
        256,
        512,
    )
    .unwrap();
    instance.device().poll(wgpu::Maintain::Wait);
    baked_env
        .save(instance.device(), instance.queue(), "env.bake")
        .unwrap();
}
