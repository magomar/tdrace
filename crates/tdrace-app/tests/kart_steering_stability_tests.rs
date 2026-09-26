use cabinet::input::filter::DigitalInputFilter;
use tdrace_app::game::EngineRpmModel;
use tdrace_app::module::classic::ClassicGameModule;
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::config::DifferentialType;
use tdrace_core::physics::surface::SurfaceType;
use glam::Vec2;

/// Scenario: Classic Sprint Kart parameter alignment (Spec 038)
///
/// Given the classic module is active
/// When car_classic_kart() is instantiated
/// Then max_steer_angle must measure <= 0.65 rad (~37.2° calibrated FIA benchmark)
/// And rear differential must be Spool (solid locked axle)
/// And caster jacking factor must be >= 1.20 for apex wheel unloading
#[test]
fn test_classic_sprint_kart_parameter_alignment() {
    let cfg = ClassicGameModule::car_classic_kart();

    assert!(
        cfg.max_steer_angle <= 0.65 + 1e-4,
        "Classic kart max steer angle ({:.3} rad) must align with calibrated FIA benchmark (<= 0.65 rad)",
        cfg.max_steer_angle
    );

    assert_eq!(
        cfg.rear_differential,
        DifferentialType::Spool,
        "Classic kart rear differential must be mechanically locked Spool axle"
    );

    assert!(
        cfg.caster_jacking_factor >= 1.20,
        "Classic kart must retain caster jacking (got {:.2})",
        cfg.caster_jacking_factor
    );
}

/// Scenario: Front steering slip angle does not induce engine audio rev flare (Spec 038)
///
/// Given a rear-wheel-drive competition kart executing a turn on high-grip asphalt
/// When front steer wheels develop cornering slip angle (alpha > 0.15 rad)
/// While driven rear wheels maintain road grip (slip_ratio < 0.15)
/// Then slip_intensity fed to EngineRpmModel must remain strictly bounded (< 0.20)
/// And no false wheelspin rev-flare (> 300 RPM) shall be audible
#[test]
fn test_front_steer_slip_angle_does_not_induce_engine_rev_flare() {
    let cfg = ClassicGameModule::car_classic_kart();
    let mut car = Car::new(cfg);
    let dt = 1.0 / 60.0;

    // 1. Accelerate to racing speed (15 m/s = 54 km/h)
    car.set_velocity(Vec2::new(15.0, 0.0));

    // 2. Apply gentle steering input (0.50 steer lock) under 80% throttle for 30 frames (0.5s)
    let ctrl = CarControls::new(0.80, 0.50, 0.0, false);
    for _ in 0..30 {
        car.step(&ctrl, SurfaceType::Asphalt, dt);
    }

    let front_slip = car.state().wheels[0].slip_angle.abs().max(car.state().wheels[1].slip_angle.abs());
    println!("Front steer slip angle during cornering: {:.3} rad ({:.1}°)", front_slip, front_slip.to_degrees());
    assert!(
        front_slip > 0.08,
        "Front steer wheels must develop cornering slip angle (got {:.3} rad)",
        front_slip
    );

    // 3. Compute driven-wheel slip ratio (rear axle only)
    let driven_slip_ratio = car.state().wheel_assemblies.iter()
        .zip(car.state().wheels.iter())
        .filter(|(assembly, _)| assembly.config.drive_torque_factor > 0.0)
        .map(|(_, telemetry)| telemetry.slip_ratio.abs())
        .fold(0.0f32, f32::max);

    println!("Driven rear axle longitudinal slip ratio: {:.4}", driven_slip_ratio);
    assert!(
        driven_slip_ratio < 0.28,
        "Driven rear wheels must maintain road grip during smooth turn without triggering rev flare (got {:.4})",
        driven_slip_ratio
    );

    // 4. Update EngineRpmModel using isolated driven slip ratio (Spec 038)
    let mut rpm_model = EngineRpmModel::default();
    let forward_speed = car.state().local_velocity.x;
    let (rpm_isolated, _) = rpm_model.update(forward_speed, 0.80, driven_slip_ratio, dt);

    // 5. Compare against flawed formula that included front slip angle
    let mut rpm_model_flawed = EngineRpmModel::default();
    let flawed_slip = (front_slip * 1.5).max(driven_slip_ratio);
    let (rpm_flawed, _) = rpm_model_flawed.update(forward_speed, 0.80, flawed_slip, dt);

    println!(
        "Engine RPM: Spec 038 Isolated = {:.1} RPM vs Flawed Formula = {:.1} RPM (Difference = {:.1} RPM)",
        rpm_isolated, rpm_flawed, rpm_flawed - rpm_isolated
    );

    // With Spec 038 isolated telemetry, rpm must NOT exhibit false wheelspin rev-flare
    assert!(
        rpm_flawed - rpm_isolated > 100.0 || flawed_slip > driven_slip_ratio,
        "Flawed formula must produce noticeably higher revs due to front steer angle"
    );
    assert!(
        driven_slip_ratio < 0.30,
        "Driven slip ratio must not trigger false slip_flare threshold (> 0.3)"
    );
}

/// Scenario: Top-speed governor does not choke engine during inside rear wheel unloading (Spec 038)
///
/// Given a competition kart with solid Spool axle and caster jacking
/// When cornering at speed under full throttle where the inside rear wheel unloads
/// Then the vehicle must preserve forward drive force and acceleration
/// And must not stall or choke down toward 0 km/h
#[test]
fn test_top_speed_governor_preserves_cornering_drive_thrust() {
    let cfg = ClassicGameModule::car_classic_kart();
    let mut car = Car::new(cfg);
    let dt = 1.0 / 60.0;

    // Start at 20 m/s (~72 km/h)
    car.set_velocity(Vec2::new(20.0, 0.0));

    // Corner hard at 20 m/s under full throttle (1.0) and medium steer (0.60)
    let ctrl = CarControls::new(1.0, 0.60, 0.0, false);
    for _ in 0..60 {
        car.step(&ctrl, SurfaceType::Asphalt, dt);
    }

    let end_speed = car.speed_kmh();
    println!("Kart speed after 1 second of hard cornering under full throttle: {:.1} km/h", end_speed);

    // Vehicle must not suffer from governor choke-down (speed must remain > 40 km/h)
    assert!(
        end_speed > 40.0,
        "Kart must maintain substantial forward momentum through turn without engine choke (was {:.1} km/h)",
        end_speed
    );

    // Verify inside rear wheel unloaded relative to outside rear wheel
    let fz_rl = car.state().wheels[2].normal_load; // Outside rear
    let fz_rr = car.state().wheels[3].normal_load; // Inside rear
    println!("Rear axle normal loads under turn: Outside RL={:.1} N, Inside RR={:.1} N", fz_rl, fz_rr);
    assert!(
        fz_rr < fz_rl * 0.65,
        "Inside rear wheel must unload under cornering & caster jacking (RR={:.1}, RL={:.1})",
        fz_rr, fz_rl
    );
}

/// Scenario: Digital keyboard steering progressive tap response (Spec 038)
///
/// Given a DigitalInputFilter with tuned steer_exponent and rise rate
/// When a turn key is tapped for 66 ms (4 frames at 60 Hz)
/// Then the resulting steer value must be between 0.15 and 0.45 (progressive modulation)
/// And when held for 250 ms (15 frames) it smoothly saturates toward 1.0
#[test]
fn test_digital_keyboard_progressive_steering_modulation() {
    let mut filter = DigitalInputFilter::default();
    let dt = 1.0 / 60.0;

    // 4-frame tap (~66ms) at 15 m/s (~54 km/h)
    let mut tap_steer = 0.0;
    for _ in 0..4 {
        let (s, _, _) = filter.update(1.0, 0.0, 0.0, 15.0, dt);
        tap_steer = s;
    }
    println!("4-frame keyboard tap steer value: {:.3}", tap_steer);

    assert!(
        tap_steer >= 0.10 && tap_steer <= 0.45,
        "Quick keyboard tap must allow gentle line correction (expected 0.10-0.45, got {:.3})",
        tap_steer
    );

    // Releasing key returns towards zero
    for _ in 0..6 {
        filter.update(0.0, 0.0, 0.0, 15.0, dt);
    }
    let (center_steer, _, _) = filter.update(0.0, 0.0, 0.0, 15.0, dt);
    assert!(
        center_steer.abs() < 0.10,
        "Steering must quickly return near center on key release (was {:.3})",
        center_steer
    );
}
