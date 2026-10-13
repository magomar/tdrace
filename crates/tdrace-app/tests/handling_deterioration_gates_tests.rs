//! Simulation-based handling deterioration regression gates.
//!
//! Any parameter change impacting vehicle dynamics, powertrain torque splits,
//! differentials, brake force, or tire compounds must satisfy these non-negotiable
//! performance and handling gates before being merged.

use glam::Vec2;
use tdrace_app::catalog::find_model_by_id;
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;

const DT: f32 = 1.0 / 60.0;

fn measure_braking(cfg: &CarConfig, surface: SurfaceType) -> (f32, f32) {
    let mut car = Car::new(cfg.clone()).with_pose(Vec2::ZERO, 0.0);
    car.set_velocity(Vec2::new(100.0 / 3.6, 0.0));
    let ctrl = CarControls::new(0.0, 0.0, 1.0, false);
    let mut dist = 0.0f32;
    let mut time = 0.0f32;
    for _ in 0..600 {
        if car.state().speed < 0.5 {
            break;
        }
        let v_prev = car.state().speed;
        car.step(&ctrl, surface, DT);
        dist += (v_prev + car.state().speed) * 0.5 * DT;
        time += DT;
    }
    let avg_decel_g = (100.0 / 3.6) / (time.max(0.01) * 9.81);
    (avg_decel_g, dist)
}

fn measure_sweeper(cfg: &CarConfig, surface: SurfaceType, speed_kmh: f32) -> (f32, f32) {
    let mut car = Car::new(cfg.clone()).with_pose(Vec2::ZERO, 0.0);
    car.set_velocity(Vec2::new(speed_kmh / 3.6, 0.0));
    let ctrl = CarControls::new(1.0, 1.0, 0.0, false);
    let mut sum_speed = 0.0f32;
    let mut sum_yaw = 0.0f32;
    for _ in 0..180 {
        car.step(&ctrl, surface, DT);
        sum_speed += car.state().speed;
        sum_yaw += car.state().angular_velocity.abs();
    }
    let heading_deg = car.state().angle.abs().to_degrees();
    let avg_speed = sum_speed / 180.0;
    let avg_yaw = sum_yaw / 180.0;
    let radius = if avg_yaw > 1e-3 { avg_speed / avg_yaw } else { 999.0 };
    (heading_deg, radius)
}

fn measure_steady_turn(cfg: &CarConfig, surface: SurfaceType, speed_kmh: f32) -> f32 {
    let mut car = Car::new(cfg.clone()).with_pose(Vec2::ZERO, 0.0);
    car.set_velocity(Vec2::new(speed_kmh / 3.6, 0.0));
    let ctrl = CarControls::new(0.50, 0.75, 0.0, false);
    for _ in 0..120 {
        car.step(&ctrl, surface, DT);
    }
    let speed = car.state().speed;
    let yaw = car.state().angular_velocity.abs();
    if yaw > 1e-3 { speed / yaw } else { 999.0 }
}

fn measure_exit_rear_slip(cfg: &CarConfig, surface: SurfaceType, speed_kmh: f32) -> f32 {
    let mut car = Car::new(cfg.clone()).with_pose(Vec2::ZERO, 0.0);
    car.set_velocity(Vec2::new(speed_kmh / 3.6, 0.0));
    let ctrl = CarControls::new(1.0, 0.75, 0.0, false);
    let mut sum_r_slip = 0.0f32;
    for _ in 0..120 {
        car.step(&ctrl, surface, DT);
        let r = (car.state().wheels[2].slip_angle.abs() + car.state().wheels[3].slip_angle.abs()) * 0.5 * 57.2958;
        sum_r_slip += r;
    }
    sum_r_slip / 120.0
}

#[test]
fn test_handling_gate_turn_in_agility_and_sweeper_radius() {
    let test_cases = [
        ("autocross_petersen_buggy1600_t3", 3, 90.0, 45.0),
        ("autocross_bohemia_veloce_t4", 4, 80.0, 55.0),
        ("autocross_petersen_superbuggy_t5", 5, 80.0, 55.0),
        ("rally_gallia_lyon_t4", 4, 80.0, 55.0),
        ("rally_nordic_valkyrie_t6", 6, 80.0, 55.0),
    ];

    for (car_id, tier, min_heading_deg, max_radius_m) in test_cases {
        let model = find_model_by_id(car_id)
            .unwrap_or_else(|| panic!("Car model {} not found", car_id));
        let cfg = model.to_car_config();

        let (heading_deg, radius) = measure_sweeper(&cfg, SurfaceType::Dirt, 70.0);

        println!(
            "[Handling Gate] {} (T{}): SweeperDeg={:.1}° (req >={:.1}°), Radius={:.1}m (req <={:.1}m)",
            model.name, tier, heading_deg, min_heading_deg, radius, max_radius_m
        );

        assert!(
            heading_deg >= min_heading_deg,
            "Handling Gate Failed for {}: Heading rotation ({:.1}°) below required minimum ({:.1}°). Car is understeering!",
            model.name, heading_deg, min_heading_deg
        );
        assert!(
            radius <= max_radius_m,
            "Handling Gate Failed for {}: Turning radius ({:.1}m) exceeds allowable maximum ({:.1}m). Car is pushing wide!",
            model.name, radius, max_radius_m
        );
    }
}

#[test]
fn test_handling_gate_steady_state_turning_radius() {
    let test_cars = [
        ("autocross_petersen_buggy1600_t3", 42.0),
        ("autocross_bohemia_veloce_t4", 42.0),
        ("autocross_petersen_superbuggy_t5", 52.0),
        ("rally_gallia_lyon_t4", 45.0),
        ("rally_nordic_valkyrie_t6", 55.0),
    ];

    for (car_id, max_rad) in test_cars {
        let model = find_model_by_id(car_id).unwrap();
        let cfg = model.to_car_config();
        let radius = measure_steady_turn(&cfg, SurfaceType::Dirt, 60.0);

        println!("[Handling Gate] {} Steady-State Radius: {:.1}m (req <= {:.1}m)", model.name, radius, max_rad);

        assert!(
            radius <= max_rad,
            "Handling Gate Failed for {}: Steady turn radius ({:.1}m) exceeds {:.1}m!",
            model.name, radius, max_rad
        );
    }
}

#[test]
fn test_handling_gate_anti_snap_spin_corner_exit() {
    let test_cars = [
        ("autocross_petersen_buggy1600_t3", 35.0),
        ("autocross_bohemia_veloce_t4", 35.0),
        ("autocross_petersen_superbuggy_t5", 40.0),
        ("rally_gallia_lyon_t4", 35.0),
        ("rally_nordic_valkyrie_t6", 35.0),
    ];

    for (car_id, max_allowed_slip) in test_cars {
        let model = find_model_by_id(car_id).unwrap();
        let cfg = model.to_car_config();
        let r_slip = measure_exit_rear_slip(&cfg, SurfaceType::Dirt, 60.0);

        println!("[Handling Gate] {} Exit Rear Slip: {:.1}° (max allowed: {:.1}°)", model.name, r_slip, max_allowed_slip);

        assert!(
            r_slip <= max_allowed_slip,
            "Handling Gate Failed for {}: Corner exit rear slip ({:.1}°) exceeds stability threshold ({:.1}°). Car suffers power snap-oversteer!",
            model.name, r_slip, max_allowed_slip
        );
    }
}

#[test]
fn test_handling_gate_competition_braking_efficiency() {
    // Autocross competition platforms (100% loose dirt tracks)
    let ax_cars = [
        "autocross_bohemia_veloce_t4",
        "autocross_petersen_superbuggy_t5",
    ];
    for car_id in ax_cars {
        let model = find_model_by_id(car_id).unwrap();
        let cfg = model.to_car_config();
        let (decel_dirt, stop_dirt) = measure_braking(&cfg, SurfaceType::Dirt);

        println!(
            "[Handling Gate] {} Dirt Braking: {:.2}G ({:.1}m)",
            model.name, decel_dirt, stop_dirt
        );

        assert!(
            decel_dirt >= 1.20,
            "Handling Gate Failed for {}: Dirt deceleration ({:.2}G) below 1.20G minimum",
            model.name, decel_dirt
        );
        assert!(
            stop_dirt <= 32.0,
            "Handling Gate Failed for {}: Dirt stopping distance ({:.1}m) exceeds 32.0m",
            model.name, stop_dirt
        );
    }

    // Rallycross competition platforms (mixed asphalt and dirt circuits)
    let rx_cars = [
        "rally_gallia_lyon_t4",
        "rally_nordic_valkyrie_t6",
    ];
    for car_id in rx_cars {
        let model = find_model_by_id(car_id).unwrap();
        let cfg = model.to_car_config();
        let (decel_dirt, stop_dirt) = measure_braking(&cfg, SurfaceType::Dirt);
        let (decel_asphalt, stop_asphalt) = measure_braking(&cfg, SurfaceType::Asphalt);

        println!(
            "[Handling Gate] {} RX Braking: Dirt={:.2}G ({:.1}m), Asphalt={:.2}G ({:.1}m)",
            model.name, decel_dirt, stop_dirt, decel_asphalt, stop_asphalt
        );

        assert!(
            decel_dirt >= 1.20,
            "Handling Gate Failed for {}: Dirt deceleration ({:.2}G) below 1.20G minimum",
            model.name, decel_dirt
        );
        assert!(
            stop_dirt <= 32.0,
            "Handling Gate Failed for {}: Dirt stopping distance ({:.1}m) exceeds 32.0m",
            model.name, stop_dirt
        );
        assert!(
            decel_asphalt >= 1.15,
            "Handling Gate Failed for {}: Asphalt deceleration ({:.2}G) below 1.15G minimum",
            model.name, decel_asphalt
        );
        assert!(
            stop_asphalt <= 34.0,
            "Handling Gate Failed for {}: Asphalt stopping distance ({:.1}m) exceeds 34.0m",
            model.name, stop_asphalt
        );
    }
}
