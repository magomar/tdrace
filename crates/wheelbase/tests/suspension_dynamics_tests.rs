//! Suspension dynamics regression test suite (Spec 076).
//!
//! Verifies closed-form 4-corner suspension compliance, six distinct suspension archetypes,
//! dynamic camber kinematics, bump-stop bottoming events, and diagonal chassis jacking.

use glam::Vec2;
use wheelbase::config::{CarConfig, SuspensionConfig};
use wheelbase::{Car, CarControls, SurfaceType};

/// Scenario 1: Kart rigid chassis transfers kerb shock directly to diagonal wheels
#[test]
fn test_kart_rigid_chassis_diagonal_jacking_on_kerb() {
    let dt = 1.0 / 120.0;
    let mut cfg = CarConfig::kart();
    cfg.suspension = SuspensionConfig::rigid_kart();

    let mut car = Car::new(cfg);
    car.state.velocity = Vec2::new(15.0, 0.0);
    car.state.speed = 15.0;

    let baseline_ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    car.step(&baseline_ctrl, SurfaceType::Asphalt, dt);
    let baseline_rr_fz = car.state.wheels[3].normal_load;

    // Front-left wheel clips a 40mm apex kerb
    car.state.wheel_elevations = [0.04, 0.0, 0.0, 0.0];
    car.step(&baseline_ctrl, SurfaceType::Asphalt, dt);

    let fl_telemetry = car.state.suspension[0];
    let rr_fz = car.state.wheels[3].normal_load;

    // FL stroke must clamp to bump travel limit (0.008m)
    assert!(
        fl_telemetry.deflection <= 0.008 + 1e-4,
        "Kart FL deflection ({:.4}m) must be clamped to max bump travel <= 0.008m",
        fl_telemetry.deflection
    );

    // Diagonal jacking unloads RR by >= 40%
    let rr_drop_pct = (baseline_rr_fz - rr_fz) / baseline_rr_fz;
    assert!(
        rr_drop_pct >= 0.40,
        "Kart kerb strike on FL must unload RR by >= 40% (baseline={:.1}N, kerb={:.1}N, drop={:.1}%)",
        baseline_rr_fz, rr_fz, rr_drop_pct * 100.0
    );
}

/// Scenario 2: GT4 MacPherson strut exhibits camber loss and understeer under heavy roll
#[test]
fn test_gt4_macpherson_camber_loss_and_grip_degradation() {
    let dt = 1.0 / 120.0;
    let mut cfg = CarConfig::sports_car();
    cfg.mass = 1320.0;
    cfg.cg_height = 0.40;
    cfg.suspension = SuspensionConfig::macpherson_strut();

    let mut car = Car::new(cfg);
    car.state.velocity = Vec2::new(26.0, 0.0);
    car.state.speed = 26.0;

    // Steer left to produce roll to the right (positive roll angle)
    let steer_ctrl = CarControls::new(0.65, -0.40, 0.0, false);
    for _ in 0..60 {
        car.step(&steer_ctrl, SurfaceType::Asphalt, dt);
    }

    let roll_deg = car.state.roll_angle.to_degrees().abs();
    assert!(
        roll_deg >= 2.5,
        "GT4 MacPherson cornering must produce >= 2.5 deg roll, got {:.2} deg",
        roll_deg
    );

    // Outside front wheel is FR (index 1) in a left turn
    let fr_susp = car.state.suspension[1];
    let static_camber = car.config.suspension.front.static_camber;
    let delta_camber = fr_susp.dynamic_camber - static_camber;
    let roll_rad = car.state.roll_angle.abs();
    let d_gamma_d_phi = delta_camber / roll_rad;

    assert!(
        d_gamma_d_phi >= 0.30,
        "GT4 MacPherson outside tire dynamic camber must degrade with d_gamma/d_phi >= 0.30, got {:.3}",
        d_gamma_d_phi
    );

    // Grip degradation relative to upright baseline
    let baseline_mu = 1.0 - 1.8 * static_camber * static_camber;
    let camber = fr_susp.dynamic_camber;
    let delta_gamma_loss = delta_camber.max(0.0);
    let dynamic_mu = 1.0 - 1.8 * camber * camber - 1.0 * delta_gamma_loss;
    let grip_loss_pct = (baseline_mu - dynamic_mu) / baseline_mu;

    assert!(
        grip_loss_pct >= 0.025,
        "GT4 MacPherson camber loss must reduce grip by >= 2.5%, got {:.2}%",
        grip_loss_pct * 100.0
    );
}

/// Scenario 3: GT3 Double Wishbone preserves contact patch across high-G sweepers
#[test]
fn test_gt3_double_wishbone_camber_preservation() {
    let dt = 1.0 / 120.0;
    let mut cfg = CarConfig::sports_car();
    cfg.suspension = SuspensionConfig::double_wishbone();

    let mut car = Car::new(cfg);
    car.state.velocity = Vec2::new(26.0, 0.0);
    car.state.speed = 26.0;

    let steer_ctrl = CarControls::new(0.65, -0.40, 0.0, false);
    for _ in 0..60 {
        car.step(&steer_ctrl, SurfaceType::Asphalt, dt);
    }

    let fr_susp = car.state.suspension[1];
    let static_camber = car.config.suspension.front.static_camber;
    let delta_camber = fr_susp.dynamic_camber - static_camber;
    let roll_rad = car.state.roll_angle.abs();
    let d_gamma_d_phi = delta_camber / roll_rad.max(1e-4);

    assert!(
        d_gamma_d_phi.abs() <= 0.15,
        "GT3 Double Wishbone outside dynamic camber must remain compensated with |d_gamma/d_phi| <= 0.15, got {:.3}",
        d_gamma_d_phi
    );

    let baseline_mu = 1.0 - 1.8 * static_camber * static_camber;
    let camber = fr_susp.dynamic_camber;
    let delta_gamma_loss = delta_camber.max(0.0);
    let dynamic_mu = 1.0 - 1.8 * camber * camber - 1.0 * delta_gamma_loss;
    let grip_loss_pct = (baseline_mu - dynamic_mu) / baseline_mu;

    assert!(
        grip_loss_pct <= 0.005,
        "GT3 Double Wishbone must maintain grip within 0.5% of optimal baseline, got {:.2}%",
        grip_loss_pct * 100.0
    );
}

/// Scenario 4: Extreme Off-Road absorbs jump landings without bottoming out
#[test]
fn test_extreme_offroad_jump_landing_absorption() {
    let dt = 1.0 / 120.0;
    let mut cfg = CarConfig::sand_rail();
    cfg.suspension = SuspensionConfig::long_travel_offroad();

    let mut car = Car::new(cfg);
    car.state.elevation = 0.02; // Close to ground
    car.state.vertical_velocity = -2.5; // Touchdown impact speed

    let ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    car.step(&ctrl, SurfaceType::Dirt, dt);

    for (i, susp) in car.state.suspension.iter().enumerate() {
        assert!(
            susp.deflection <= 0.240 + 1e-4,
            "Offroad corner {} deflection ({:.3}m) must remain within max bump travel (0.240m)",
            i, susp.deflection
        );
        assert!(
            !susp.bottomed_out,
            "Offroad corner {} must not bottom out upon landing with -2.5m/s",
            i
        );
    }
}

/// Scenario 5: Hypercar bottoming out on sausage kerb triggers load spike and telemetry
#[test]
fn test_hypercar_bottoming_out_on_sausage_kerb() {
    let dt = 1.0 / 120.0;
    let mut cfg = CarConfig::sports_car();
    cfg.suspension = SuspensionConfig::pushrod_inboard();

    let mut car = Car::new(cfg);
    car.state.velocity = Vec2::new(40.0, 0.0);
    car.state.speed = 40.0;

    let ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    car.step(&ctrl, SurfaceType::Asphalt, dt);
    let baseline_fl_fz = car.state.wheels[0].normal_load;

    // Front-left strikes a +0.06m sausage kerb
    car.state.wheel_elevations = [0.06, 0.0, 0.0, 0.0];
    car.step(&ctrl, SurfaceType::Asphalt, dt);

    let fl_susp = car.state.suspension[0];
    let kerb_fl_fz = car.state.wheels[0].normal_load;

    assert!(
        fl_susp.bottomed_out,
        "Hypercar striking 60mm sausage kerb with 25mm bump travel must report bottomed_out = true"
    );

    let spike_ratio = (kerb_fl_fz - baseline_fl_fz) / baseline_fl_fz;
    assert!(
        spike_ratio >= 3.0,
        "Hypercar bottoming out must spike corner normal load by >= 300% (baseline={:.1}N, spike={:.1}N, ratio={:.1}%)",
        baseline_fl_fz, kerb_fl_fz, spike_ratio * 100.0
    );
}

/// Scenario 6: Solid live axle coupled roll kinematics
#[test]
fn test_solid_live_axle_coupled_camber_kinematics() {
    let dt = 1.0 / 120.0;
    let mut cfg = CarConfig::stock_car_ta1();
    cfg.suspension = SuspensionConfig::solid_live_axle();

    let mut car = Car::new(cfg);
    car.state.velocity = Vec2::new(20.0, 0.0);
    car.state.speed = 20.0;

    let ctrl = CarControls::new(1.0, 0.0, 0.0, false);

    // Strike curb unilaterally on rear-left (index 2)
    car.state.wheel_elevations = [0.0, 0.0, 0.04, 0.0];
    car.step(&ctrl, SurfaceType::Asphalt, dt);

    let rl_camber = car.state.suspension[2].dynamic_camber;
    let rr_camber = car.state.suspension[3].dynamic_camber;

    // For a live axle, beam tilt produces equal and opposite dynamic camber changes
    assert!(
        (rl_camber + rr_camber).abs() < 1e-3,
        "Solid live axle rear cambers must be antisymmetric: RL={:.4}, RR={:.4}",
        rl_camber, rr_camber
    );
}
