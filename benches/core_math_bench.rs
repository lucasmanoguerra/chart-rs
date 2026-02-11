use chart_rs::core::{LinearScale, Viewport};
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_linear_scale_round_trip(c: &mut Criterion) {
    let viewport = Viewport::new(1920, 1080);
    let scale = LinearScale::new(0.0, 10_000.0).expect("valid scale");

    c.bench_function("linear_scale_round_trip", |b| {
        b.iter(|| {
            let px = scale
                .domain_to_pixel(4_321.123, viewport)
                .expect("to pixel");
            let _ = scale.pixel_to_domain(px, viewport).expect("from pixel");
        })
    });
}

criterion_group!(benches, bench_linear_scale_round_trip);
criterion_main!(benches);
