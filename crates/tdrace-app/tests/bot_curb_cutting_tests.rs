use tdrace_app::ai::bot_harness::sample_bot;
use tdrace_app::ai::{DriverTier, DrivingStyle};
use tdrace_core::catalog::official_track;
use tdrace_core::physics::car::Car;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;

#[test]
fn test_no_weaving_on_straight() {
    let track = official_track("classic", "oval_speedway");
    let cfg = CarConfig::sports_car();
    let dt = 1.0 / 120.0;

    for tier in [DriverTier::Rookie, DriverTier::Contender, DriverTier::Legend] {
        let mut bot = sample_bot(DrivingStyle::Balanced, tier, 1234);
        let start_sample = track.spline.sample_at_distance(0.0);
        let mut car = Car::new(cfg).with_pose(start_sample.point, start_sample.tangent.y.atan2(start_sample.tangent.x));
        car.state.velocity = start_sample.tangent * 25.0;
        car.state.speed = 25.0;

        let mut steer_reversals = 0;
        let mut last_steer_sign = 0.0f32;

        for _ in 0..360 { // 3 seconds on straight
            let controls = bot.compute_controls(&car, &track, &[], dt);
            car.step(&controls, SurfaceType::Asphalt, dt);

            if controls.steer.abs() > 0.015 {
                let sign = controls.steer.signum();
                if last_steer_sign != 0.0 && sign != last_steer_sign {
                    steer_reversals += 1;
                }
                last_steer_sign = sign;
            }
        }
        println!("{tier:?}: steer reversals in 3s on straight: {steer_reversals}");
        assert!(steer_reversals <= 2, "{tier:?} has excessive steer reversals ({steer_reversals})");
    }
}

#[test]
fn test_tier_5_cuts_apex_curbs() {
    let track = official_track("classic", "classic_grand_prix");
    let cfg = CarConfig::sports_car();
    let dt = 1.0 / 120.0;

    // Run Tier 5 Aggressive
    let mut bot_t5 = sample_bot(DrivingStyle::Aggressive, DriverTier::Legend, 42);
    let spawn = track.grid_positions[0];
    let mut car_t5 = Car::new(cfg).with_pose(spawn.position, spawn.angle);

    let mut t5_curb_hits = 0;
    let mut t5_max_curb_depth = 0.0f32;

    // Run for 1 lap (~90-100 seconds = ~12000 steps on gt_coastal_grand_prix)
    for _ in 0..12000 {
        let controls = bot_t5.compute_controls(&car_t5, &track, &[], dt);
        car_t5.step(&controls, SurfaceType::Asphalt, dt);
        let proj = track.spline.project_point(car_t5.state.position);

        let half_w = proj.track_width * 0.5;
        let dist_past_edge = proj.lateral_offset.abs() - half_w;
        if proj.is_on_curb || (dist_past_edge > 0.05 && dist_past_edge < 1.35) {
            t5_curb_hits += 1;
            t5_max_curb_depth = t5_max_curb_depth.max(dist_past_edge);
        }
    }

    // Run Tier 1 Rookie (Balanced)
    let mut bot_t1 = sample_bot(DrivingStyle::Balanced, DriverTier::Rookie, 42);
    let mut car_t1 = Car::new(cfg).with_pose(spawn.position, spawn.angle);

    let mut t1_curb_hits = 0;
    for _ in 0..12000 {
        let controls = bot_t1.compute_controls(&car_t1, &track, &[], dt);
        car_t1.step(&controls, SurfaceType::Asphalt, dt);
        let proj = track.spline.project_point(car_t1.state.position);
        let half_w = proj.track_width * 0.5;
        let dist_past_edge = proj.lateral_offset.abs() - half_w;
        if proj.is_on_curb || (dist_past_edge > 0.05 && dist_past_edge < 1.35) {
            t1_curb_hits += 1;
        }
    }

    println!("T5 Aggressive curb hits: {t5_curb_hits}, max depth: {t5_max_curb_depth:.2}m");
    println!("T1 Balanced curb hits: {t1_curb_hits}");

    assert!(t5_curb_hits > 0, "Tier 5 Aggressive must cut curbs on corners with curbs");
    assert!(t5_max_curb_depth > 0.2, "Tier 5 Aggressive must ride at least 0.2m onto curbs ({t5_max_curb_depth:.2}m)");
    assert!(t5_curb_hits >= t1_curb_hits * 5, "Tier 5 must cut significantly more curbs than Tier 1 ({t5_curb_hits} vs {t1_curb_hits})");
}
