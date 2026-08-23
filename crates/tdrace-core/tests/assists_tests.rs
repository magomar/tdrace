use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::config::{CarConfig, DriverAssistsConfig};
use tdrace_core::physics::surface::SurfaceType;

#[test]
fn test_tcs_prevents_power_oversteer_wheelspin() {
    let dt = 1.0 / 60.0;

    // 1. With TCS enabled (Arcade preset)
    let mut car_tcs = Car::new(CarConfig::sports_car());
    // First build speed
    for _ in 0..200 {
        car_tcs.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }
    // Now perform high-throttle aggressive turn
    let mut tcs_ever_active = false;
    let turn_ctrl = CarControls::new(1.0, 0.8, 0.0, false);
    for _ in 0..60 {
        car_tcs.step(&turn_ctrl, SurfaceType::Asphalt, dt);
        if car_tcs.state.tcs_active {
            tcs_ever_active = true;
        }
    }
    let sideslip_tcs = car_tcs.state.sideslip_angle.abs();
    let speed_tcs = car_tcs.speed_kmh();
    println!("TCS Car: speed={:.1} km/h, sideslip={:.3} rad, tcs_ever_active={}", speed_tcs, sideslip_tcs, tcs_ever_active);

    // 2. With TCS disabled (Raw preset)
    let mut raw_cfg = CarConfig::sports_car();
    raw_cfg.assists = DriverAssistsConfig::raw();
    let mut car_raw = Car::new(raw_cfg);
    for _ in 0..200 {
        car_raw.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }
    for _ in 0..60 {
        car_raw.step(&turn_ctrl, SurfaceType::Asphalt, dt);
    }
    let sideslip_raw = car_raw.state.sideslip_angle.abs();
    let speed_raw = car_raw.speed_kmh();
    println!("Raw Car: speed={:.1} km/h, sideslip={:.3} rad, tcs_active={}", speed_raw, sideslip_raw, car_raw.state.tcs_active);

    assert!(
        tcs_ever_active,
        "TCS must actively engage during high-throttle cornering"
    );
    assert!(
        sideslip_tcs < sideslip_raw,
        "TCS must keep vehicle sideslip lower than unassisted raw car (tcs={sideslip_tcs}, raw={sideslip_raw})"
    );
}

#[test]
fn test_esc_stabilizes_yaw_rate_on_sudden_steering_reversals() {
    let dt = 1.0 / 60.0;

    // 1. Accelerate car
    let mut car_esc = Car::new(CarConfig::sports_car());
    for _ in 0..240 {
        car_esc.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }
    assert!(car_esc.speed_kmh() > 50.0);

    // Perform violent steering snap left then right (slalom maneuver)
    for _ in 0..15 {
        car_esc.step(&CarControls::new(0.6, -1.0, 0.0, false), SurfaceType::Asphalt, dt);
    }
    for _ in 0..15 {
        car_esc.step(&CarControls::new(0.6, 1.0, 0.0, false), SurfaceType::Asphalt, dt);
    }
    // Release steering
    for _ in 0..30 {
        car_esc.step(&CarControls::new(0.8, 0.0, 0.0, false), SurfaceType::Asphalt, dt);
    }

    let yaw_rate_esc = car_esc.state.angular_velocity.abs();
    let sideslip_esc = car_esc.state.sideslip_angle.abs();
    println!("ESC Slalom Recovery: yaw_rate={:.3} rad/s, sideslip={:.3} rad", yaw_rate_esc, sideslip_esc);

    assert!(yaw_rate_esc < 1.2, "ESC must dampen residual yaw oscillation ({yaw_rate_esc} rad/s)");
    assert!(sideslip_esc < 0.25, "ESC must restore directional stability ({sideslip_esc} rad)");
}

#[test]
fn test_handbrake_bypass_allows_intentional_power_drifts() {
    let dt = 1.0 / 60.0;
    let mut car = Car::new(CarConfig::sports_car());

    // Build speed
    for _ in 0..180 {
        car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }

    // Pull handbrake and steer hard
    let mut entered_drift = false;
    for _ in 0..35 {
        let ctrl = CarControls::new(0.8, 1.0, 0.0, true);
        car.step(&ctrl, SurfaceType::Asphalt, dt);
        if car.state.is_drifting {
            entered_drift = true;
        }
    }

    assert!(
        entered_drift,
        "Handbrake bypass must allow driver to initiate intentional drift despite active assists"
    );
    assert!(
        car.state.drift_score > 0.0,
        "Drift score must accumulate during handbrake slide"
    );
}

#[test]
fn test_assist_presets_configuration() {
    let arcade = DriverAssistsConfig::arcade();
    let sport = DriverAssistsConfig::sport();
    let raw = DriverAssistsConfig::raw();

    assert!(arcade.tcs_enabled);
    assert!(arcade.esc_enabled);
    assert!(arcade.counter_steer_assist_enabled);

    assert!(sport.tcs_enabled);
    assert!(sport.tcs_slip_threshold > arcade.tcs_slip_threshold); // Sport allows more slip

    assert!(!raw.tcs_enabled);
    assert!(!raw.esc_enabled);
    assert!(!raw.counter_steer_assist_enabled);
}
