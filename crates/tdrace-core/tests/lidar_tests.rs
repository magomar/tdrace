use std::time::Instant;
use glam::Vec2;
use tdrace_core::lidar::{LidarConfig, LidarHitType, LidarScanner};
use tdrace_core::physics::{Car, CarConfig};
use tdrace_core::track::geometry::{Obstacle, WallBarrier, BarrierType};
use tdrace_core::track::presets::{classic_grand_prix, oval_speedway};

#[test]
fn test_lidar_raycast_accuracy_and_normal() {
    let mut track = classic_grand_prix();
    // Add an exact test wall at x = 10.0 from y=-10 to y=10 with normal pointing -X
    track.geometry.inner_walls.push(WallBarrier::new(
        Vec2::new(10.0, -10.0),
        Vec2::new(10.0, 10.0),
        BarrierType::Concrete,
    ));

    let scanner = LidarScanner::new(LidarConfig {
        num_rays: 1,
        fov_radians: 0.0,
        max_range: 50.0,
        offset_forward: 0.0,
        angle_offset: 0.0, // pointing directly +X
    });

    let car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    let hits = scanner.scan(&car, &track, &[]);

    assert_eq!(hits.len(), 1);
    let hit = hits[0];
    assert_eq!(hit.hit_type, LidarHitType::TrackWall);
    assert!(
        (hit.distance - 10.0).abs() < 1e-3,
        "Distance must be 10.0m, got {}",
        hit.distance
    );
    assert_eq!(hit.normalized_distance, 10.0 / 50.0);
    assert!((hit.hit_point.x - 10.0).abs() < 1e-3);
    assert!((hit.hit_point.y - 0.0).abs() < 1e-3);
}

#[test]
fn test_lidar_distance_normalization_and_miss() {
    let track = classic_grand_prix();
    let scanner = LidarScanner::new(LidarConfig::surround_32());
    let car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);

    let hits = scanner.scan(&car, &track, &[]);
    for (i, hit) in hits.iter().enumerate() {
        assert!(
            hit.normalized_distance >= 0.0 && hit.normalized_distance <= 1.0,
            "Normalized distance for ray {} must be in [0, 1], got {}",
            i,
            hit.normalized_distance
        );
        assert!(hit.distance >= 0.0 && hit.distance <= scanner.config.max_range);
    }
}

#[test]
fn test_lidar_obstacle_and_opponent_detection() {
    let mut track = oval_speedway();
    track.geometry.obstacles.push(Obstacle::circle(
        99,
        Vec2::new(8.0, 0.0),
        1.0,
        "Test Obstacle",
    ));

    let scanner = LidarScanner::new(LidarConfig::gym_carracing_19());
    let host = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    let mut opp = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(15.0, 0.0), 0.0);
    opp.state.velocity = Vec2::new(10.0, 0.0);

    let hits = scanner.scan(&host, &track, &[opp.clone()]);

    // Center ray (index 9 for 19-ray 180 FOV scanner) points directly forward
    let center_idx = 9;
    let hit = hits[center_idx];
    assert_eq!(hit.hit_type, LidarHitType::Obstacle);
    // Sensor offset = 1.2m, circle front = 7.0m -> distance = 5.8m
    assert!((hit.distance - 5.8).abs() < 0.5, "Obstacle hit distance should be ~5.8m, got {}", hit.distance);

    // Remove the obstacle, now center ray should hit opponent car at ~12.55m
    track.geometry.obstacles.clear();
    let hits_opp = scanner.scan(&host, &track, &[opp]);
    let hit_opp = hits_opp[center_idx];
    assert_eq!(hit_opp.hit_type, LidarHitType::OpponentCar);
    assert!((hit_opp.distance - 12.55).abs() < 0.8, "Opponent distance should be ~12.55m, got {}", hit_opp.distance);
    assert_eq!(hit_opp.relative_velocity, Vec2::new(10.0, 0.0));
}

#[test]
fn test_lidar_throughput_benchmark_exceeds_1m_rays_per_sec() {
    let track = classic_grand_prix();
    let scanner = LidarScanner::new(LidarConfig::surround_32());
    let host = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(50.0, 0.0), 0.0);

    // 8 dynamic opponent cars on track
    let opponents = vec![
        Car::new(CarConfig::sports_car()).with_pose(Vec2::new(60.0, 2.0), 0.0),
        Car::new(CarConfig::sports_car()).with_pose(Vec2::new(70.0, -2.0), 0.0),
        Car::new(CarConfig::sports_car()).with_pose(Vec2::new(80.0, 0.0), 0.0),
        Car::new(CarConfig::sports_car()).with_pose(Vec2::new(40.0, 1.0), 0.0),
        Car::new(CarConfig::sports_car()).with_pose(Vec2::new(30.0, -1.0), 0.0),
        Car::new(CarConfig::sports_car()).with_pose(Vec2::new(20.0, 0.0), 0.0),
        Car::new(CarConfig::sports_car()).with_pose(Vec2::new(100.0, 3.0), 0.0),
        Car::new(CarConfig::sports_car()).with_pose(Vec2::new(110.0, -3.0), 0.0),
    ];

    let mut buffer = vec![Default::default(); 32];

    // Warmup
    for _ in 0..1_000 {
        scanner.scan_into(&host, &track, &opponents, &mut buffer);
    }

    let iterations = if cfg!(debug_assertions) { 5_000 } else { 50_000 };
    let total_rays = iterations * 32;

    let start = Instant::now();
    for _ in 0..iterations {
        scanner.scan_into(&host, &track, &opponents, &mut buffer);
    }
    let elapsed = start.elapsed();
    let seconds = elapsed.as_secs_f64();
    let rays_per_sec = (total_rays as f64) / seconds;

    println!("⚡ LIDAR Benchmark: {:.2} rays/second ({:.2} ns/ray)", rays_per_sec, (elapsed.as_nanos() as f64) / (total_rays as f64));
    let target = if cfg!(debug_assertions) { 50_000.0 } else { 1_000_000.0 };
    assert!(
        rays_per_sec >= target,
        "LIDAR throughput must exceed target {:.2}, achieved {:.2}",
        target,
        rays_per_sec
    );
}
