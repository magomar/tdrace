use std::time::Instant;
use tdrace_core::physics::{Car, CarConfig, CarControls, SurfaceType};

fn main() {
    println!("============================================================");
    println!("🏁 TDRace Physics Throughput Benchmark");
    println!("============================================================");

    let mut car = Car::new(CarConfig::sports_car());
    let controls = [
        CarControls::new(1.0, 0.0, 0.0, false),
        CarControls::new(1.0, 0.5, 0.0, false),
        CarControls::new(0.5, -0.5, 0.0, true),
        CarControls::new(0.0, 0.0, 1.0, false),
    ];
    let surfaces = [
        SurfaceType::Asphalt,
        SurfaceType::Curb,
        SurfaceType::Grass,
        SurfaceType::Sand,
    ];

    let warmup_steps = 100_000;
    println!("Warming up with {} steps...", warmup_steps);
    for i in 0..warmup_steps {
        let ctrl = &controls[i % controls.len()];
        let surf = surfaces[i % surfaces.len()];
        car.step(ctrl, surf, 1.0 / 60.0);
    }

    let bench_steps = 2_000_000;
    println!("Running {} physics steps...", bench_steps);

    let start = Instant::now();
    for i in 0..bench_steps {
        let ctrl = &controls[i % controls.len()];
        let surf = surfaces[i % surfaces.len()];
        car.step(ctrl, surf, 1.0 / 60.0);
    }
    let elapsed = start.elapsed();
    let seconds = elapsed.as_secs_f64();
    let steps_per_sec = (bench_steps as f64) / seconds;
    let nanoseconds_per_step = (elapsed.as_nanos() as f64) / (bench_steps as f64);

    println!("------------------------------------------------------------");
    println!("Elapsed Time:        {:.4} s", seconds);
    println!("Throughput:          {:.2} steps/second", steps_per_sec);
    println!("Latency per step:    {:.2} ns/step", nanoseconds_per_step);
    println!("Target Bar:          > 500,000 steps/second");
    if steps_per_sec >= 500_000.0 {
        println!("Status:              ✅ PASS (Exceeds target by {:.1}x)", steps_per_sec / 500_000.0);
    } else {
        println!("Status:              ❌ FAIL (Below 500k target)");
    }
    println!("============================================================");

    assert!(steps_per_sec >= 500_000.0, "Throughput must exceed 500,000 steps/sec");
}
