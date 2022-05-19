use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use nextclade::align::seed_alignment::{create_stripes, SeedMatch};

pub fn bench_create_stripes(c: &mut Criterion) {
  let seed_matches = black_box(vec![
    SeedMatch {
      qry_pos: 5,
      ref_pos: 10,
      score: 0,
    },
    SeedMatch {
      qry_pos: 20,
      ref_pos: 30,
      score: 0,
    },
  ]);

  let terminal_bandwidth = black_box(5);
  let excess_bandwidth = black_box(2);
  let qry_len = black_box(30);
  let ref_len = black_box(40);

  let mut group = c.benchmark_group("create_stripes");
  group.throughput(Throughput::Bytes(qry_len as u64));
  group.bench_function("create_stripes", |b| {
    b.iter(|| create_stripes(&seed_matches, qry_len, ref_len, terminal_bandwidth, excess_bandwidth))
  });
  group.finish();
}

criterion_group!(benches, bench_create_stripes);
criterion_main!(benches);
