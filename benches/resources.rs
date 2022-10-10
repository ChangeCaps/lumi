use criterion::{black_box, criterion_group, criterion_main, Criterion};

use lumi::resources::Resources;

pub fn resources(c: &mut Criterion) {
    let mut resources = Resources::new();

    c.bench_function("insert usize", |b| {
        b.iter(|| {
            resources.insert(black_box(0));
        })
    });

    c.bench_function("get usize", |b| {
        b.iter(|| {
            resources.get::<usize>();
        })
    });

    c.bench_function("remove usize", |b| {
        b.iter(|| {
            resources.remove::<usize>();
        })
    });
}

pub fn resources_key(c: &mut Criterion) {
    let mut resources = Resources::new();

    c.bench_function("insert_key usize", |b| {
        b.iter(|| {
            resources.insert_key(black_box(0usize), black_box(0usize));
        })
    });

    c.bench_function("get_key usize", |b| {
        b.iter(|| {
            resources.get_key::<usize>(black_box(&0usize));
        })
    });

    c.bench_function("remove_key usize", |b| {
        b.iter(|| {
            resources.remove_key::<usize>(black_box(&0usize));
        })
    });
}

criterion_group!(benches, resources, resources_key);
criterion_main!(benches);
