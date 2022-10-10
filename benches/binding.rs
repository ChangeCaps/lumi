use criterion::{black_box, criterion_group, criterion_main, Criterion};

use lumi::binding::{BindingGroupKey, BindingKind, BindingsState};

pub fn state(c: &mut Criterion) {
    let mut bindings_state = BindingsState::default();
    let key = BindingGroupKey::new("test_with_long_name", BindingKind::UniformBuffer);

    c.bench_function("insert", |b| {
        b.iter(|| {
            bindings_state.get_mut::<Vec<usize>>(black_box(&key));
        })
    });
}

criterion_group!(benches, state);
criterion_main!(benches);
