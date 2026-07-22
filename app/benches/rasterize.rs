//! Benchmarks for `scope::rasterize` across raster resolutions and sweep
//! densities (sparse vs. full-360-radial sweeps).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rustywx::model::{Product, synthetic_sweep};
use rustywx::scope::rasterize;

const MAX_RANGE_KM: f32 = 230.0;
const TDBZ_KERNEL_SIZE: usize = 9; // matches settings::TdbzKernel::Default
const GATES_PER_RADIAL: usize = 912; // (230 - 2.125) / 0.25, realistic super-res gate count

fn bench_rasterize(c: &mut Criterion) {
    let mut group = c.benchmark_group("rasterize");

    for &n_radials in &[30usize, 360usize] {
        let sweep = synthetic_sweep(n_radials, GATES_PER_RADIAL);
        for &size_px in &[128usize, 256, 512, 1024] {
            group.bench_function(format!("radials={n_radials}/size={size_px}"), |b| {
                b.iter(|| {
                    black_box(rasterize(
                        black_box(&sweep),
                        Product::Reflectivity,
                        size_px,
                        MAX_RANGE_KM,
                        TDBZ_KERNEL_SIZE,
                        None,  // cc_sweep
                        false, // cc_gate_enabled
                        0.80,  // cc_gate_threshold
                        false, // refl_floor_enabled
                        7.0,   // refl_floor_dbz
                        false, // vel_sd_censor_enabled
                        7.0,   // vel_sd_threshold
                    ))
                });
            });
        }
    }

    group.finish();
}

criterion_group!(benches, bench_rasterize);
criterion_main!(benches);
