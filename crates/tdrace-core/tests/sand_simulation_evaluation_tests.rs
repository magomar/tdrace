//! Empirical Simulation and Diagnostic Suite: Sand Surface Dynamics & Steering Evaluation
//!
//! Investigates and quantifies vehicle behavior on Sand surfaces:
//! 1. Longitudinal thrust vs rolling resistance saturation
//! 2. Lateral steering authority and friction ellipse starvation
//! 3. End-to-end simulation on dynamically generated hypothetical tracks
//! 4. End-to-end simulation on official preset sand circuits (Sahara, Glamis, Atacama)

use glam::Vec2;
use tdrace_core::physics::car::{normalize_angle, Car, CarControls};
use tdrace_core::physics::sim::{
    run_path_simulation, PathSimulationStatus, SimPath, DEFAULT_SIMULATION_DT,
};
use tdrace_core::physics::{CarConfig, SurfaceType};
use tdrace_core::track::presets::{atacama_sand_basin, glamis_sand_dunes, sahara_dune_crossing};
use tdrace_core::track::Track;

const G_ACCEL: f32 = 9.80665;

#[test]
fn test_sand_vs_asphalt_dirt_acceleration_and_force_breakdown() {
    let dt = DEFAULT_SIMULATION_DT;
    let full_throttle = CarControls::accelerate();

    let test_surfaces = [
        SurfaceType::Asphalt,
        SurfaceType::Dirt,
        SurfaceType::Grass,
        SurfaceType::DeepSand,
    ];

    println!("\n==================================================================================");
    println!("🔬 1. LONGITUDINAL ACCELERATION & RESISTANCE FORCE DYNAMICS");
    println!("==================================================================================");

    for &surf in &test_surfaces {
        let mut car = Car::new(CarConfig::sports_car());
        let total_mass = car.config.mass;
        let normal_load_total = total_mass * G_ACCEL;

        let mu = surf.friction_coefficient();
        let rr_multiplier = surf.rolling_resistance_multiplier();
        let effective_rr_coeff = car.config.rolling_resistance_coefficient * rr_multiplier;
        let static_rr_force = effective_rr_coeff * normal_load_total;

        // Driven axle (RWD) static weight distribution: cg_to_front / wheelbase
        let rear_load_static = normal_load_total * (car.config.cg_to_front / car.config.wheelbase);
        let max_possible_traction_n = mu * rear_load_static;

        // Step simulation for 3.0 seconds
        let mut speed_1s = 0.0;
        let mut speed_2s = 0.0;
        let mut speed_3s = 0.0;
        let mut max_speed = 0.0;

        for step in 1..=360 {
            car.step(&full_throttle, surf, dt);
            let spd = car.speed_kmh();
            if spd > max_speed {
                max_speed = spd;
            }
            if step == 120 {
                speed_1s = spd;
            } else if step == 240 {
                speed_2s = spd;
            } else if step == 360 {
                speed_3s = spd;
            }
        }

        let actual_rear_traction: f32 = car.state.wheels[2..=3]
            .iter()
            .map(|w| w.longitudinal_force.max(0.0))
            .sum();

        println!(
            "{:<8} | mu={:.2} | RR mult={:>4.1}x ({:>5.0} N) | Max Drive Cap={:>5.0} N | Actual Trac={:>5.0} N | Net Fx={:>+5.0} N",
            format!("{:?}", surf),
            mu,
            rr_multiplier,
            static_rr_force,
            max_possible_traction_n,
            actual_rear_traction,
            actual_rear_traction - static_rr_force
        );
        println!(
            "         Speed Progression: 1.0s = {:>5.1} km/h | 2.0s = {:>5.1} km/h | 3.0s = {:>5.1} km/h | Peak = {:>5.1} km/h",
            speed_1s, speed_2s, speed_3s, max_speed
        );
        println!("----------------------------------------------------------------------------------");

        if surf == SurfaceType::DeepSand {
            // Assert and document the physical failure condition:
            // Rolling resistance exceeds maximum possible tire traction on DeepSand!
            assert!(
                static_rr_force > max_possible_traction_n,
                "Physical defect: Rolling resistance ({:.0} N) must exceed RWD tire traction cap ({:.0} N) on DeepSand",
                static_rr_force, max_possible_traction_n
            );
            assert!(
                speed_3s < 3.0,
                "Sports car on DeepSand is paralyzed (speed after 3s was {:.2} km/h)",
                speed_3s
            );
        } else if surf == SurfaceType::Asphalt {
            assert!(speed_3s > 50.0);
        }
    }
}

#[test]
fn test_sand_steering_authority_and_friction_ellipse_starvation() {
    let dt = DEFAULT_SIMULATION_DT;

    println!("\n==================================================================================");
    println!("🔄 2. STEERING AUTHORITY & COMBINED SLIP FRICTION ELLIPSE DYNAMICS");
    println!("   Evaluating a vehicle entering a 30m turn at 40 km/h with 100% steering lock");
    println!("==================================================================================");

    let test_surfaces = [
        SurfaceType::Asphalt,
        SurfaceType::Dirt,
        SurfaceType::DeepSand,
    ];

    let v_init_kmh = 40.0;
    let v_init_mps = v_init_kmh / 3.6;

    for &surf in &test_surfaces {
        let mut car = Car::new(CarConfig::sports_car());
        car.state.position = Vec2::ZERO;
        car.state.angle = 0.0; // Facing east (+X)
        car.state.velocity = Vec2::new(v_init_mps, 0.0);
        car.state.speed = v_init_mps;

        // Apply 100% left steer with maintenance throttle (50%)
        let steer_ctrl = CarControls {
            throttle: 0.50,
            steer: -1.0, // Full left
            brake: 0.0,
            handbrake: false,
            reverse: false,
        };

        let mut peak_yaw_rate = 0.0f32;
        let mut peak_lat_accel = 0.0f32;
        let mut speed_after_1s = 0.0f32;
        let mut heading_change_1s = 0.0f32;
        let mut front_fy_sum = 0.0f32;
        let mut front_fx_sum = 0.0f32;
        let mut front_slip_angle_sum = 0.0f32;

        for step in 1..=120 {
            car.step(&steer_ctrl, surf, dt);

            let yaw_rate = car.state.angular_velocity.abs().to_degrees();
            if yaw_rate > peak_yaw_rate {
                peak_yaw_rate = yaw_rate;
            }

            let lat_accel = car.state.acceleration_local.y.abs() / G_ACCEL;
            if lat_accel > peak_lat_accel {
                peak_lat_accel = lat_accel;
            }

            let f_fx = (car.state.wheels[0].longitudinal_force.abs()
                + car.state.wheels[1].longitudinal_force.abs())
                * 0.5;
            let f_fy = (car.state.wheels[0].lateral_force.abs()
                + car.state.wheels[1].lateral_force.abs())
                * 0.5;
            let f_slip = (car.state.wheels[0].slip_angle.abs()
                + car.state.wheels[1].slip_angle.abs())
                * 0.5;

            front_fx_sum += f_fx;
            front_fy_sum += f_fy;
            front_slip_angle_sum += f_slip.to_degrees();

            if step == 120 {
                speed_after_1s = car.speed_kmh();
                heading_change_1s = car.state.angle.abs().to_degrees();
            }
        }

        let avg_front_fx = front_fx_sum / 120.0;
        let avg_front_fy = front_fy_sum / 120.0;
        let avg_front_slip = front_slip_angle_sum / 120.0;

        println!(
            "{:<8} | Peak Ay={:>4.2}g | Peak Yaw Rate={:>5.1}°/s | 1s Heading Turn={:>5.1}° | 1s Exit Speed={:>4.1} km/h",
            format!("{:?}", surf),
            peak_lat_accel,
            peak_yaw_rate,
            heading_change_1s,
            speed_after_1s
        );
        println!(
            "         Front Tire Forces: Avg Fx (Rolling Drag)={:>5.0} N | Avg Fy (Steering Grip)={:>5.0} N | Slip Angle={:>4.1}°",
            avg_front_fx, avg_front_fy, avg_front_slip
        );
        println!("----------------------------------------------------------------------------------");

        if surf == SurfaceType::DeepSand {
            // On DeepSand, the car plows straight ahead because lateral grip is starved by longitudinal drag
            assert!(
                heading_change_1s < 5.0,
                "DeepSand severely stunts yaw turn authority (got {:.1} deg in 1s vs Asphalt >30 deg)",
                heading_change_1s
            );
            assert!(
                peak_lat_accel < 0.15,
                "DeepSand lateral acceleration severely compromised ({:.2}g)",
                peak_lat_accel
            );
            assert!(
                avg_front_fy < 800.0,
                "Front steering grip collapsed by rolling resistance starvation ({:.0} N)",
                avg_front_fy
            );
        } else if surf == SurfaceType::Asphalt {
            assert!(peak_lat_accel > 0.65);
            assert!(heading_change_1s > 30.0);
        }
    }
}

#[test]
fn test_dynamically_built_tracks_end_to_end_simulation() {
    let config = CarConfig::sports_car();

    println!("\n==================================================================================");
    println!("🏁 3. DYNAMICALLY BUILT TRACKS: END-TO-END AUTOMOTIVE SIMULATION");
    println!("==================================================================================");

    // Track 1: Straight Path with Sequential Turns & S-Chicane (~410m)
    let straight_with_turns = SimPath::straight_with_turns();
    println!("Course A: Dynamic Straight Path with Sequential 90° Turns & Chicane (Length: {:.1}m)", straight_with_turns.total_length);

    let res_asphalt_path = run_path_simulation(&config, SurfaceType::Asphalt, &straight_with_turns, 30.0, DEFAULT_SIMULATION_DT);
    let res_dirt_path = run_path_simulation(&config, SurfaceType::Dirt, &straight_with_turns, 35.0, DEFAULT_SIMULATION_DT);
    let res_sand_path = run_path_simulation(&config, SurfaceType::DeepSand, &straight_with_turns, 20.0, DEFAULT_SIMULATION_DT);

    println!(
        "  Asphalt : Status={:<18} | Completed={:>5.1}% ({:>5.1}m in {:>4.1}s) | Avg Spd={:>4.1} km/h | Peak Spd={:>5.1} km/h | Max Lat Dev={:.2}m",
        res_asphalt_path.status.as_str(),
        res_asphalt_path.completion_pct,
        res_asphalt_path.distance_traveled_m,
        res_asphalt_path.elapsed_time_s,
        res_asphalt_path.avg_speed_kmh,
        res_asphalt_path.peak_speed_kmh,
        res_asphalt_path.max_cross_track_error_m
    );
    println!(
        "  Dirt    : Status={:<18} | Completed={:>5.1}% ({:>5.1}m in {:>4.1}s) | Avg Spd={:>4.1} km/h | Peak Spd={:>5.1} km/h | Max Lat Dev={:.2}m",
        res_dirt_path.status.as_str(),
        res_dirt_path.completion_pct,
        res_dirt_path.distance_traveled_m,
        res_dirt_path.elapsed_time_s,
        res_dirt_path.avg_speed_kmh,
        res_dirt_path.peak_speed_kmh,
        res_dirt_path.max_cross_track_error_m
    );
    println!(
        "  DeepSand: Status={:<18} | Completed={:>5.1}% ({:>5.1}m in {:>4.1}s) | Avg Spd={:>4.1} km/h | Peak Spd={:>5.1} km/h | Max Lat Dev={:.2}m",
        res_sand_path.status.as_str(),
        res_sand_path.completion_pct,
        res_sand_path.distance_traveled_m,
        res_sand_path.elapsed_time_s,
        res_sand_path.avg_speed_kmh,
        res_sand_path.peak_speed_kmh,
        res_sand_path.max_cross_track_error_m
    );
    if let Some(err) = &res_sand_path.failure_reason {
        println!("            Failure Diagnostic: {}", err);
    }
    println!("----------------------------------------------------------------------------------");

    assert_eq!(res_asphalt_path.status, PathSimulationStatus::Completed);
    assert_eq!(res_dirt_path.status, PathSimulationStatus::Completed);
    assert_eq!(res_sand_path.status, PathSimulationStatus::StuckInSand);

    // Track 2: Closed Hypothetical Circuit (~711m)
    let circuit = SimPath::hypothetical_circuit();
    println!("Course B: Closed Hypothetical Grand Prix Circuit (Length: {:.1}m)", circuit.total_length);

    let res_asphalt_circ = run_path_simulation(&config, SurfaceType::Asphalt, &circuit, 40.0, DEFAULT_SIMULATION_DT);
    let res_dirt_circ = run_path_simulation(&config, SurfaceType::Dirt, &circuit, 45.0, DEFAULT_SIMULATION_DT);
    let res_sand_circ = run_path_simulation(&config, SurfaceType::DeepSand, &circuit, 20.0, DEFAULT_SIMULATION_DT);

    println!(
        "  Asphalt : Status={:<18} | Completed={:>5.1}% ({:>5.1}m in {:>4.1}s) | Avg Spd={:>4.1} km/h | Peak Spd={:>5.1} km/h",
        res_asphalt_circ.status.as_str(),
        res_asphalt_circ.completion_pct,
        res_asphalt_circ.distance_traveled_m,
        res_asphalt_circ.elapsed_time_s,
        res_asphalt_circ.avg_speed_kmh,
        res_asphalt_circ.peak_speed_kmh
    );
    println!(
        "  Dirt    : Status={:<18} | Completed={:>5.1}% ({:>5.1}m in {:>4.1}s) | Avg Spd={:>4.1} km/h | Peak Spd={:>5.1} km/h",
        res_dirt_circ.status.as_str(),
        res_dirt_circ.completion_pct,
        res_dirt_circ.distance_traveled_m,
        res_dirt_circ.elapsed_time_s,
        res_dirt_circ.avg_speed_kmh,
        res_dirt_circ.peak_speed_kmh
    );
    println!(
        "  DeepSand: Status={:<18} | Completed={:>5.1}% ({:>5.1}m in {:>4.1}s) | Avg Spd={:>4.1} km/h | Peak Spd={:>5.1} km/h",
        res_sand_circ.status.as_str(),
        res_sand_circ.completion_pct,
        res_sand_circ.distance_traveled_m,
        res_sand_circ.elapsed_time_s,
        res_sand_circ.avg_speed_kmh,
        res_sand_circ.peak_speed_kmh
    );
    if let Some(err) = &res_sand_circ.failure_reason {
        println!("            Failure Diagnostic: {}", err);
    }
    println!("----------------------------------------------------------------------------------");

    assert_eq!(res_asphalt_circ.status, PathSimulationStatus::Completed);
    assert_eq!(res_dirt_circ.status, PathSimulationStatus::Completed);
    assert_eq!(res_sand_circ.status, PathSimulationStatus::StuckInSand);
}

#[test]
fn test_real_game_sand_circuits_end_to_end_behavior() {
    println!("\n==================================================================================");
    println!("🏜️ 4. OFFICIAL GAME SAND PRESETS: SAHARA, GLAMIS, & ATACAMA SIMULATION");
    println!("==================================================================================");

    let tracks: [(&str, Track); 3] = [
        ("Sahara Dune Crossing", sahara_dune_crossing()),
        ("Glamis Imperial Sand Dunes", glamis_sand_dunes()),
        ("Atacama Sand Basin", atacama_sand_basin()),
    ];

    let dt = DEFAULT_SIMULATION_DT;

    for (name, track) in tracks {
        let spline = &track.spline;
        let total_len = spline.total_length();

        // Count how many waypoints are PackedSand vs other surfaces
        let sand_wp_count = track
            .spline
            .waypoints
            .iter()
            .filter(|w| w.surface == Some(SurfaceType::PackedSand))
            .count();
        let total_wps = track.spline.waypoints.len();

        // Run closed-loop car simulation along the spline
        let start_sample = spline.sample_at_distance(0.0);
        let start_angle = start_sample.tangent.y.atan2(start_sample.tangent.x);
        let mut car = Car::new(CarConfig::sports_car()).with_pose(start_sample.point, start_angle);

        let mut progress_dist = 0.0f32;
        let mut sim_time = 0.0f32;
        let max_sim_time = 20.0f32;
        let mut peak_speed_kmh = 0.0f32;

        while sim_time < max_sim_time {
            let car_pos = car.state.position;
            let car_angle = car.state.angle;

            if car.speed_kmh() > peak_speed_kmh {
                peak_speed_kmh = car.speed_kmh();
            }

            // Project onto spline
            let proj = spline.project_point(car_pos);
            if proj.progress_distance > progress_dist {
                progress_dist = proj.progress_distance;
            }

            // Target 15m ahead
            let target_sample = spline.sample_at_distance((proj.progress_distance + 15.0) % total_len);
            let to_target = target_sample.point - car_pos;
            let target_heading = to_target.y.atan2(to_target.x);
            let heading_error = normalize_angle(target_heading - car_angle);

            let steer = -(heading_error * 2.5).clamp(-1.0, 1.0);
            let ctrl = CarControls {
                throttle: 1.0,
                steer,
                brake: 0.0,
                handbrake: false,
                reverse: false,
            };

            // Sample surface per wheel from actual track geometry
            let wheel_surfaces = track.sample_car_surfaces(&car);
            car.step_per_wheel(&ctrl, wheel_surfaces, dt);
            sim_time += dt;

            // Early exit if vehicle is completely stuck at start
            if sim_time > 4.0 && progress_dist < 4.0 {
                break;
            }
        }

        let completion_pct = (progress_dist / total_len * 100.0).clamp(0.0, 100.0);
        println!(
            "Track: {:<28} | Total Length: {:>6.1}m | PackedSand Waypoints: {}/{}",
            name, total_len, sand_wp_count, total_wps
        );
        println!(
            "  Default Surface: {:?} | Simulated in {:.1}s -> Progress: {:.1}m ({:.1}%) | Peak Speed: {:.1} km/h",
            track.default_surface, sim_time, progress_dist, completion_pct, peak_speed_kmh
        );

        // Under Spec 025 PackedSand bifurcation, tracks are fully drivable
        assert!(
            progress_dist > 30.0,
            "Track {} should be playable on PackedSand (traveled {:.1}m)",
            name, progress_dist
        );
        assert!(
            peak_speed_kmh > 20.0,
            "Track {} cars should reach racing speeds on PackedSand (peak {:.1} km/h)",
            name, peak_speed_kmh
        );
        println!("----------------------------------------------------------------------------------");
    }
}
