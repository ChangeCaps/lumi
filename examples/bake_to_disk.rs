use lumi_bake::{BakeInstance, BakedEnvironment};

fn main() {
    let instance = BakeInstance::new().unwrap();
    let baked_env =
        BakedEnvironment::open_from_eq(instance.device(), instance.queue(), "env.hdr").unwrap();
    instance.device().poll(wgpu::Maintain::Wait);
    baked_env
        .save(instance.device(), instance.queue(), "env.bake")
        .unwrap();
}
