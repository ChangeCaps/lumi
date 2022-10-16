use criterion::{black_box, criterion_group, criterion_main, Criterion};

use lumi::{id::IdMap, prelude::NodeId};

pub fn id_map(c: &mut Criterion) {
    let id = NodeId::new();
    let mut map = IdMap::<usize>::new();
    map.insert(id, 0);

    c.bench_function("insert usize", |b| {
        b.iter(|| {
            map.insert(id, black_box(0));
        })
    });

    c.bench_function("get usize", |b| {
        b.iter(|| {
            map.get(black_box(&id));
        })
    });

    c.bench_function("remove usize", |b| {
        b.iter(|| {
            map.remove(&id);
        })
    });
}

criterion_group!(benches, id_map);
criterion_main!(benches);
