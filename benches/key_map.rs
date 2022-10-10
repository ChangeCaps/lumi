use criterion::{black_box, criterion_group, criterion_main, Criterion};

use lumi::key_map::KeyMap;

pub fn small_key(c: &mut Criterion) {
    let key = 0usize;
    let mut key_map = KeyMap::<usize>::new();

    c.bench_function("insert_small_key", |b| {
        b.iter(|| {
            key_map.insert(black_box(key), black_box(0usize));
        })
    });

    c.bench_function("get_small_key", |b| {
        b.iter(|| {
            key_map.get(black_box(&key));
        })
    });

    c.bench_function("remove_small_key", |b| {
        b.iter(|| {
            key_map.remove(black_box(&key));
        })
    });
}

pub fn large_key(c: &mut Criterion) {
    let key = [0usize; 1024];
    let mut key_map = KeyMap::<usize>::new();

    c.bench_function("insert_large_key", |b| {
        b.iter(|| {
            key_map.insert(black_box(key), black_box(0usize));
        })
    });

    c.bench_function("get_large_key", |b| {
        b.iter(|| {
            key_map.get(black_box(&key));
        })
    });

    c.bench_function("remove_large_key", |b| {
        b.iter(|| {
            key_map.remove(black_box(&key));
        })
    });
}

criterion_group!(benches, small_key, large_key);
criterion_main!(benches);
