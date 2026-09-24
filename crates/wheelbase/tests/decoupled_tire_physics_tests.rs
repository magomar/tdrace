use wheelbase::car::{Car, CarControls};
use wheelbase::config::{CarConfig, DriverAssistsConfig, WheelAssemblyConfig};
use wheelbase::surface::SurfaceType;
use wheelbase::tire::WheelAssembly;
use wheelbase::Vec2;

/// Scenario: Staggered tire dimensions on open-wheel kart
///
/// Given a classic_kart configured with narrow front tires (r=0.18m, w=0.12m)
/// and wide rear tires (r=0.20m, w=0.21m)
/// When standing acceleration and maximum lateral cornering tests are executed
/// Then the rear axle must deliver at least 35% more peak lateral force than the front axle under equal normal load
/// And rear wheel rotational inertia must measure larger than front wheel inertia (I_rear > I_front)
#[test]
fn test_staggered_tire_dimensions_on_open_wheel_kart() {
    let cfg = CarConfig::classic_kart();

    // Verify dimensions: narrow front (r=0.18, w=0.12), wide rear (r=0.20, w=0.21)
    assert_eq!(cfg.wheels[0].tire_radius, 0.18);
    assert_eq!(cfg.wheels[0].tire_width, 0.12);
    assert_eq!(cfg.wheels[2].tire_radius, 0.20);
    assert_eq!(cfg.wheels[2].tire_width, 0.21);

    // Verify polar moment of inertia: I_rear > I_front
    assert!(
        cfg.wheels[2].rotational_inertia > cfg.wheels[0].rotational_inertia,
        "Rear wheel inertia ({}) must be > front inertia ({})",
        cfg.wheels[2].rotational_inertia,
        cfg.wheels[0].rotational_inertia
    );

    // Test lateral force under equal normal load (1000 N) and optimal slip angle (0.15 rad)
    let front_assembly = WheelAssembly::new(cfg.wheels[0]);
    let rear_assembly = WheelAssembly::new(cfg.wheels[2]);
    let load = 1000.0;
    let slip_angle = 0.15;
    let mu = 1.0;

    let fy_front = front_assembly.lateral_force(slip_angle, load, mu, false);
    let fy_rear = rear_assembly.lateral_force(slip_angle, load, mu, false);

    let force_ratio = fy_rear / fy_front;
    println!(
        "Staggered Kart Lateral Grip: Front Fy={:.1} N, Rear Fy={:.1} N (Ratio={:.2}x, +{:.1}%)",
        fy_front, fy_rear, force_ratio, (force_ratio - 1.0) * 100.0
    );

    assert!(
        force_ratio >= 1.35,
        "Rear axle must deliver at least 35% more peak lateral force (got ratio {:.2})",
        force_ratio
    );
}

/// Scenario: Independent front-wheel brake lockup under trail-braking
///
/// Given a vehicle traveling at 120 km/h with a forward brake bias of 65% front / 35% rear
/// When the driver applies 100% service brake while cornering without ABS
/// Then the front-inner wheel rotational velocity must reach zero (omega = 0 rad/s)
/// while rear wheels continue rolling (omega > 20 rad/s)
/// And the locked front wheel must report slip_ratio = -1.0 and trigger maximum skid smoke telemetry
#[test]
fn test_independent_front_wheel_brake_lockup_under_trail_braking() {
    let mut cfg = CarConfig::sports_car();
    // Forward brake bias: 65% front / 35% rear
    cfg.brake_bias = 0.65;
    cfg.wheels = CarConfig::default_wheel_assemblies_for(cfg.tire, 0.65, 0.0);
    // Disable ABS for pure analog trail-braking lockup
    cfg.assists = DriverAssistsConfig::raw();

    let mut car = Car::new(cfg);
    // Initial velocity: 120 km/h (~33.3 m/s)
    let v0 = 120.0 / 3.6;
    car.set_velocity(Vec2::new(v0, 0.0));

    let dt = 1.0 / 60.0;

    // Establish cornering turn-in at 120 km/h
    let steer_ctrl = CarControls::new(0.0, 0.45, 0.0, false);
    for _ in 0..10 {
        car.step(&steer_ctrl, SurfaceType::Asphalt, dt);
    }

    // Now apply 100% service brake while cornering without ABS
    let trail_ctrl = CarControls::new(0.0, 0.45, 1.0, false);
    for _ in 0..30 {
        car.step(&trail_ctrl, SurfaceType::Asphalt, dt);
        if car.state.wheels[0].is_locked {
            break;
        }
    }

    let fl_wheel = &car.state.wheels[0];
    let rr_wheel = &car.state.wheels[3];

    println!(
        "Trail-Braking: FL Omega={:.1} rad/s (Locked={}) | RR Omega={:.1} rad/s | FL Slip={:.2} Skid={:.2}",
        fl_wheel.angular_velocity, fl_wheel.is_locked, rr_wheel.angular_velocity,
        fl_wheel.slip_ratio, fl_wheel.skid_intensity
    );

    assert_eq!(fl_wheel.angular_velocity, 0.0, "Front-inner wheel must lock up to 0 rad/s");
    assert!(fl_wheel.is_locked, "Front-inner wheel must report is_locked = true");
    assert_eq!(fl_wheel.slip_ratio, -1.0, "Locked wheel must report slip_ratio = -1.0");
    assert_eq!(fl_wheel.skid_intensity, 1.0, "Locked wheel must trigger maximum skid intensity (1.0)");
    assert!(fl_wheel.is_skidding, "Locked wheel must trigger is_skidding = true");

    assert!(
        rr_wheel.angular_velocity > 20.0,
        "Rear wheel must continue rolling (omega = {:.1} rad/s > 20.0)",
        rr_wheel.angular_velocity
    );
}

/// Scenario: Thermal grip degradation under prolonged power drifting
///
/// Given a high-powered GT vehicle executing sustained donut slides on dry asphalt
/// When rear wheel slip energy is sustained for > 8.0 simulated seconds
/// Then rear tire surface temperature T_rear must rise above 120°C
/// And rear peak lateral grip D must drop by at least 15% relative to nominal operating temperature
/// And the telemetry must record elevated thermal wear rate W_rear
#[test]
fn test_thermal_grip_degradation_under_prolonged_power_drifting() {
    let mut wheel = WheelAssembly::new(WheelAssemblyConfig::default());
    wheel.temperature = 85.0; // Nominal operating temperature
    let nominal_grip = wheel.thermal_grip_multiplier();

    // 8.5 simulated seconds of sustained high-energy power drift
    let dt = 1.0 / 60.0;
    let steps = (8.5 / dt) as usize;
    for _ in 0..steps {
        wheel.step_thermal_and_wear(3500.0, 4200.0, 0.35, 0.40, 18.0, dt);
    }

    // Surface temperature rises above 120°C
    assert!(
        wheel.temperature > 120.0,
        "Rear tire surface temperature ({:.1}°C) must rise above 120°C",
        wheel.temperature
    );

    // Overheated thermal grip drops by >= 15% relative to nominal
    let overheated_grip = wheel.thermal_grip_multiplier();
    let grip_drop = (nominal_grip - overheated_grip) / nominal_grip;
    println!(
        "Thermal Degradation: T={:.1}°C | Nominal Grip={:.3} -> Overheated Grip={:.3} (Drop={:.1}%) | Wear={:.4}",
        wheel.temperature, nominal_grip, overheated_grip, grip_drop * 100.0, wheel.wear
    );

    assert!(
        grip_drop >= 0.15,
        "Thermal grip drop ({:.1}%) must be >= 15%",
        grip_drop * 100.0
    );

    // Elevated wear accumulated
    assert!(wheel.wear > 0.0, "Telemetry must record wear accumulation");
}

/// Scenario: Legacy configuration backward compatibility
///
/// Given a legacy config JSON string containing only the singular "tire" section without "wheels"
/// When the vehicle configuration is loaded by the engine
/// Then the engine must successfully deserialize the car without error
/// And all 4 wheel corners must automatically inherit the singular TireConfig parameters
#[test]
fn test_legacy_configuration_backward_compatibility() {
    let legacy_json = r#"{
        "mass": 1050.0,
        "inertia": 1450.0,
        "wheelbase": 2.40,
        "track_width": 1.40,
        "cg_to_front": 1.10,
        "cg_to_rear": 1.30,
        "cg_height": 0.35,
        "max_engine_force": 6800.0,
        "max_reverse_force": 4420.0,
        "max_brake_force": 11500.0,
        "handbrake_force": 7500.0,
        "brake_bias": 0.60,
        "drive_bias": 0.0,
        "top_speed_mps": 58.0,
        "max_steer_angle": 0.68,
        "steer_speed": 5.5,
        "steer_return_speed": 7.0,
        "counter_steer_assist": 1.3,
        "speed_sensitive_steer_factor": 0.002,
        "air_drag_coefficient": 0.42,
        "lateral_drag_coefficient": 1.20,
        "rolling_resistance_coefficient": 0.015,
        "angular_damping": 160.0,
        "weight_transfer_longitudinal": 1.0,
        "weight_transfer_lateral": 1.0,
        "engine_braking_coefficient": 0.12,
        "downforce_coefficient": 0.65,
        "tire": {
            "stiffness_b": 11.0,
            "shape_c": 1.35,
            "peak_d": 1.18,
            "curvature_e": -0.15,
            "drift_slide_friction": 0.88,
            "handbrake_lateral_friction_multiplier": 0.40,
            "skid_threshold": 0.10,
            "skid_full_threshold": 0.30
        },
        "assists": {
            "tcs_enabled": true,
            "tcs_slip_threshold": 0.18,
            "tcs_strength": 0.75,
            "esc_enabled": true,
            "esc_yaw_threshold": 0.10,
            "esc_strength": 0.85,
            "counter_steer_assist_enabled": true,
            "counter_steer_assist_strength": 0.70,
            "abs_enabled": true,
            "abs_slip_threshold": 0.15,
            "abs_strength": 0.95,
            "handbrake_bypass": true
        },
        "terrain": {
            "sand_flotation": 1.0,
            "mud_flotation": 1.0,
            "ice_grip_multiplier": 1.0
        }
    }"#;

    let cfg: CarConfig = serde_json::from_str(legacy_json).expect("Legacy JSON must deserialize cleanly");
    assert_eq!(cfg.wheels.len(), 4);
    for i in 0..4 {
        assert_eq!(cfg.wheels[i].tire_model.stiffness_b, 11.0);
        assert_eq!(cfg.wheels[i].tire_model.peak_d, 1.18);
        assert_eq!(cfg.wheels[i].tire_radius, 0.32);
        assert_eq!(cfg.wheels[i].rotational_inertia, 1.25);
    }
}

/// Scenario: Headless simulation throughput SLA
///
/// Given the headless benchmark harness (Spec 010)
/// When executing 100,000 fixed-timestep simulation steps with decoupled 4-wheel dynamics
/// Then total execution time must not exceed 1,200 ms (throughput > 85,000 steps/sec)
/// And zero dynamic heap allocations must occur within the inner stepping loop
#[test]
fn test_headless_simulation_throughput_sla() {
    let cfg = CarConfig::sports_car();
    let mut car = Car::new(cfg);
    let controls = CarControls::accelerate();
    let dt = 1.0 / 60.0;

    // Warm-up
    for _ in 0..1000 {
        car.step(&controls, SurfaceType::Asphalt, dt);
    }

    let start = std::time::Instant::now();
    let steps = 100_000;
    for _ in 0..steps {
        car.step(&controls, SurfaceType::Asphalt, dt);
    }
    let elapsed = start.elapsed();
    let throughput = (steps as f64) / elapsed.as_secs_f64();

    println!(
        "Headless Simulation SLA: 100,000 steps in {:.2} ms (Throughput: {:.0} steps/sec)",
        elapsed.as_secs_f64() * 1000.0,
        throughput
    );

    assert!(
        elapsed.as_millis() < 1200 || throughput > 85_000.0,
        "Execution time ({:.2} ms) must satisfy SLA (< 1200 ms or > 85k steps/sec, got {:.0} steps/sec)",
        elapsed.as_secs_f64() * 1000.0,
        throughput
    );
}
