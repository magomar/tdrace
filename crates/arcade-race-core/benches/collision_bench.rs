use std::f32::consts::PI;
use std::time::Instant;
use glam::Vec2;
use arcade_race_core::collision::{
    collide_obb_obb, resolve_all_wall_collisions, resolve_multi_car_collisions, OrientedBox,
};
use arcade_race_core::track::{create_prototypical_track, RaceDirection, TrackShape};
use wheelbase::{Car, CarConfig, CarControls, SurfaceType};

/// Regression floor for SAT checks: about 15% under the throughput measured on macOS
/// aarch64 (2026-09-28). A refactor that drops below it has made collision slower.
const FLOOR_SAT_PER_SEC: f64 = 20_000_000.0;
/// Target from `specs/constitution/ROADMAP.md` (Technical Debt: Performance).
const ROADMAP_SAT_PER_SEC: f64 = 22_000_000.0;

fn main() {
    println!("============================================================");
    println!("💥 TDRace Collision & Multi-Body Resolution Benchmark");
    println!("============================================================");

    // 1. OBB vs OBB SAT Throughput
    let box_a = OrientedBox::new(Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.9), 0.3);
    let box_b = OrientedBox::new(Vec2::new(2.5, 0.5), Vec2::new(2.0, 0.9), -0.2);

    let sat_iterations = 5_000_000;
    println!("Benchmarking SAT OBB vs OBB ({} iterations)...", sat_iterations);
    let start_sat = Instant::now();
    let mut hits = 0;
    for _ in 0..sat_iterations {
        if collide_obb_obb(&box_a, &box_b).is_some() {
            hits += 1;
        }
    }
    let elapsed_sat = start_sat.elapsed();
    let sat_ops_per_sec = (sat_iterations as f64) / elapsed_sat.as_secs_f64();
    let sat_ns_per_op = (elapsed_sat.as_nanos() as f64) / (sat_iterations as f64);
    println!("SAT Overlap Checks:  {:.2} checks/second ({:.2} ns/check)", sat_ops_per_sec, sat_ns_per_op);
    assert_eq!(hits, sat_iterations);

    // 2. Multi-Car Pileup Solver Benchmark (8 cars in continuous collision)
    let num_cars = 8;
    let mut cars: Vec<Car> = (0..num_cars)
        .map(|i| {
            let x = (i as f32) * 0.8;
            let heading = if i % 2 == 0 { 0.0 } else { PI };
            Car::new(CarConfig::sports_car()).with_pose(Vec2::new(x, 0.0), heading)
        })
        .collect();

    let pileup_steps = 100_000;
    println!("Benchmarking Multi-Car Solver (8 cars, {} steps)...", pileup_steps);
    let ctrl = CarControls::accelerate();
    let start_pileup = Instant::now();
    for _ in 0..pileup_steps {
        for c in cars.iter_mut() {
            c.step(&ctrl, SurfaceType::Asphalt, 1.0 / 60.0);
        }
        resolve_multi_car_collisions(&mut cars, 0.5, 0.3, 6);
    }
    let elapsed_pileup = start_pileup.elapsed();
    let pileup_steps_per_sec = (pileup_steps as f64) / elapsed_pileup.as_secs_f64();
    println!("8-Car Multi-Body Steps: {:.2} steps/second ({:.2} us/step)", pileup_steps_per_sec, (elapsed_pileup.as_micros() as f64) / (pileup_steps as f64));

    // 3. Wall Collision Resolution Benchmark
    // Generated track, so the bench needs no `tracks/` checkout. Before spec 049 this used
    // the official Classic Grand Prix, which has more walls: numbers are not comparable.
    let track = create_prototypical_track("classic", TrackShape::Oval, RaceDirection::Right);
    let mut wall_car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(10.0, 7.0), 0.2);
    wall_car.state.velocity = Vec2::new(20.0, 5.0);

    let wall_steps = 500_000;
    println!("Benchmarking Track Wall Collisions ({} steps)...", wall_steps);
    let start_wall = Instant::now();
    for _ in 0..wall_steps {
        resolve_all_wall_collisions(&mut wall_car, &track.geometry.outer_walls, &track.geometry.obstacles);
    }
    let elapsed_wall = start_wall.elapsed();
    let wall_checks_per_sec = (wall_steps as f64) / elapsed_wall.as_secs_f64();
    println!("Track Barrier Checks:   {:.2} checks/second ({:.2} ns/check)", wall_checks_per_sec, (elapsed_wall.as_nanos() as f64) / (wall_steps as f64));

    println!("------------------------------------------------------------");
    println!("SAT regression floor: {:.0} checks/second", FLOOR_SAT_PER_SEC);
    println!("SAT roadmap target:   {:.0} checks/second ({})", ROADMAP_SAT_PER_SEC,
        if sat_ops_per_sec >= ROADMAP_SAT_PER_SEC { "met" } else { "not met" });
    if sat_ops_per_sec >= FLOOR_SAT_PER_SEC {
        println!("Status:     ✅ PASS (floor)");
    } else {
        println!("Status:     ❌ FAIL (below floor)");
    }
    println!("============================================================");

    assert!(sat_ops_per_sec >= FLOOR_SAT_PER_SEC, "SAT throughput fell below the regression floor");
}
