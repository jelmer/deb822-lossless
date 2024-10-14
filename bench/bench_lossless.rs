use criterion::{black_box, criterion_group, criterion_main, Criterion};
use deb822_lossless::lossless::Deb822;

fn parse_deb822_benchmark(c: &mut Criterion) {
    let control_data =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/bench/Sources"))
            .expect("Could not read control file");

    c.bench_function("parse_deb822_lossless", |b| {
        b.iter(|| {
            let _deb822: Deb822 = control_data.parse().unwrap();
        });
    });
}

criterion_group!(benches, parse_deb822_benchmark);
criterion_main!(benches);
