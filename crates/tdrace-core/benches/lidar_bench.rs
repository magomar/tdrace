use std::time::Instant;
use glam::Vec2;
use tdrace_core::lidar::{LidarConfig, LidarScanner};
use tdrace_core::physics::{Car, CarConfig};
use tdrace_core::track::presets::classic_grand_prix;

fn main() {
    println!("============================================================");
    println!("📡 TDRace High-Speed LIDAR Raycasting Benchmark");
    println!("============================================================");

    let track = classic_grand_prix();
    let scanner = LidarScanner::new(LidarConfig::surround_32());
    let host = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(50.0, 0.0), 0.0);

    let opponents: Vec<Car> = (0..8)
        .map(|i| {
            let offset_x = (i as f32) * 15.0 + 10.0;
            let offset_y = if i % 2 == 0 { 2.0 } else { -2.0 };
            Car::new(CarConfig::sports_car()).with_pose(Vec2::new(50.0 + offset_x, offset_y), 0.0)
        })
        .collect();

    let sweeps = 100_000; // 100,000 sweeps * 32 rays = 3,200,000 rays
    let total_rays = sweeps * 32;

    let mut buffer = vec![Default::default(); 32];

    println!("Warming up LIDAR scanner...");
    for _ in 0..10_000 {
        scanner.scan_into(&host, &track, &opponents, &mut buffer);
    }

    println!("Running {} LIDAR sweeps ({} total rays)...", sweeps, total_rays);
    let start = Instant::now();
    for _ in 0..sweeps {
        scanner.scan_into(&host, &track, &opponents, &mut buffer);
    }
    let elapsed = start.elapsed();
    let seconds = elapsed.as_secs_f64();
    let rays_per_sec = (total_rays as f64) / seconds;
    let ns_per_ray = (elapsed.as_nanos() as f64) / (total_rays as f64);
    let sweeps_per_sec = (sweeps as f64) / seconds;

    println!("------------------------------------------------------------");
    println!("Elapsed Time:        {:.4} s", seconds);
    println!("LIDAR Sweeps/sec:    {:.2} sweeps/second", sweeps_per_sec);
    println!("Throughput:          {:.2} rays/second", rays_per_sec);
    println!("Latency per ray:     {:.2} ns/ray", ns_per_ray);
    println!("Target Bar:          > 1,000,000 rays/second");
    if rays_per_sec >= 1_000_000.0 {
        println!("Status:              ✅ PASS (Exceeds target by {:.1}x)", rays_per_sec / 1_000_000.0);
    } else {
        println!("Status:              ❌ FAIL");
    }
    println!("============================================================");

    assert!(rays_per_sec >= 1_000_000.0, "LIDAR throughput must exceed 1,000,000 rays/sec");
}
