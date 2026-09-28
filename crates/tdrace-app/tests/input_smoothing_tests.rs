//! Keyboard handling filter tests (Spec 043: five parameters, four calibrated presets).
//! Speed sensitivity and hold bleed were removed; the car's grip-aware steering replaces them.

use tdrace_app::input::{DigitalInputConfig, DigitalInputFilter, SteeringProfile};
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::config::{CarConfig, PlayerHandling};
use tdrace_core::physics::surface::SurfaceType;

#[test]
fn test_digital_input_filter_progressive_rise_and_centering() {
    let mut filter = DigitalInputFilter::default();
    let dt = 1.0 / 60.0;

    // First frame of holding steer right: should not snap instantly to 1.0
    let (s1, _, _) = filter.update(1.0, 0.0, 0.0, dt);
    assert!(s1 > 0.0, "Steering should start moving right");
    assert!(s1 < 0.15, "First frame steering must be smoothed, was {s1}");

    // Balanced reaches full input in 140 ms; 20 frames (333 ms) saturate
    for _ in 0..20 {
        filter.update(1.0, 0.0, 0.0, dt);
    }
    let (s_sat, _, _) = filter.update(1.0, 0.0, 0.0, dt);
    assert!((s_sat - 1.0).abs() < 1e-3, "Steering should saturate at 1.0, was {s_sat}");

    // Releasing key: centering (0.7x the steering time) returns quickly towards 0.0
    let (s_rel, _, _) = filter.update(0.0, 0.0, 0.0, dt);
    assert!(s_rel < 0.85, "Steering centering should be snappy on release, was {s_rel}");
    for _ in 0..10 {
        filter.update(0.0, 0.0, 0.0, dt);
    }
    let (s_zero, _, _) = filter.update(0.0, 0.0, 0.0, dt);
    assert!(s_zero.abs() < 0.05, "Steering should return near zero within 10 frames, was {s_zero}");
}

#[test]
fn test_steering_speed_setting_changes_time_to_full_input() {
    let dt = 1.0 / 120.0;
    let frames_to_full = |profile: SteeringProfile| {
        let mut filter = DigitalInputFilter::new(profile.to_config());
        let mut frames = 0;
        while filter.current_steer < 1.0 && frames < 240 {
            filter.update(1.0, 0.0, 0.0, dt);
            frames += 1;
        }
        frames
    };
    let times = SteeringProfile::PRESETS.map(frames_to_full);
    println!("Frames to full steer (120 Hz): {times:?}");
    for pair in times.windows(2) {
        assert!(pair[1] < pair[0], "each preset must steer quicker than the previous: {times:?}");
    }
}

#[test]
fn test_steering_profiles_configuration_and_cycling() {
    let mut cfg = DigitalInputConfig::default();
    assert_eq!(cfg.profile, SteeringProfile::Balanced);

    // In-race cycle: Smooth -> Balanced -> Sharp -> Raw -> Smooth; Custom goes to Smooth
    assert_eq!(SteeringProfile::Smooth.cycle(), SteeringProfile::Balanced);
    assert_eq!(SteeringProfile::Balanced.cycle(), SteeringProfile::Sharp);
    assert_eq!(SteeringProfile::Sharp.cycle(), SteeringProfile::Raw);
    assert_eq!(SteeringProfile::Raw.cycle(), SteeringProfile::Smooth);
    assert_eq!(SteeringProfile::Custom.cycle(), SteeringProfile::Smooth);

    // set_profile applies all five values; Custom keeps the current values
    cfg.set_profile(SteeringProfile::Raw);
    assert_eq!(cfg, SteeringProfile::Raw.to_config());
    cfg.steer_authority = 1.2;
    cfg.set_profile(SteeringProfile::Custom);
    assert_eq!(cfg.profile, SteeringProfile::Custom);
    assert!((cfg.steer_authority - 1.2).abs() < 1e-6);

    // Dropdown index round trip
    for p in SteeringProfile::PRESETS.into_iter().chain([SteeringProfile::Custom]) {
        assert_eq!(SteeringProfile::from_index(p.to_index()), p);
    }
    assert_eq!(SteeringProfile::from_index(99), SteeringProfile::Balanced);
}

#[test]
fn test_non_linear_center_micro_corrections() {
    let config = DigitalInputConfig {
        center_precision: 1.4,
        ..Default::default()
    };
    let mut filter = DigitalInputFilter::new(config);
    filter.current_steer = 0.3; // 30% digital input

    let (steer, _, _) = filter.update(0.3, 0.0, 0.0, 1.0 / 60.0);
    // 0.3^1.4 = ~0.185 (soft center response)
    assert!(
        steer < 0.22 && steer > 0.15,
        "Center precision 1.4 should soften 30% steer down to ~18%, was {steer}"
    );
}

#[test]
fn test_vehicle_high_speed_turn_stability_with_smoothed_input() {
    let dt = 1.0 / 60.0;
    let mut car = Car::new(CarConfig::sports_car());
    let keys = DigitalInputConfig::default();
    car.config.player = PlayerHandling::human(keys.steer_authority, keys.traction_help);
    let mut filter = DigitalInputFilter::new(keys);

    // 1. Accelerate in a straight line (240 frames = 4 seconds)
    for _ in 0..240 {
        let (_, throttle, _) = filter.update(0.0, 1.0, 0.0, dt);
        car.step(&CarControls::new(throttle, 0.0, 0.0, false), SurfaceType::Asphalt, dt);
    }
    assert!(car.speed_kmh() > 30.0, "Car must reach speed, was {:.1} km/h", car.speed_kmh());

    // 2. Lane change / gentle turn: hold steer right for 20 frames, then center for 20 frames
    for _ in 0..20 {
        let (steer, throttle, _) = filter.update(1.0, 0.8, 0.0, dt);
        car.step(&CarControls::new(throttle, steer, 0.0, false), SurfaceType::Asphalt, dt);
    }
    for _ in 0..20 {
        let (steer, throttle, _) = filter.update(0.0, 0.8, 0.0, dt);
        car.step(&CarControls::new(throttle, steer, 0.0, false), SurfaceType::Asphalt, dt);
    }

    let sideslip = car.state.sideslip_angle.abs();
    let yaw_rate = car.state.angular_velocity.abs();
    println!(
        "High-speed smoothed turn: speed={:.1} km/h, sideslip={:.3} rad, yaw_rate={:.3} rad/s",
        car.speed_kmh(), sideslip, yaw_rate
    );
    assert!(sideslip < 0.35, "Vehicle sideslip must remain controlled ({sideslip} rad)");
    assert!(yaw_rate < 1.5, "Vehicle yaw rate must not exceed stability limit ({yaw_rate} rad/s)");
}

#[test]
fn test_keyboard_progressive_brake_tap_vs_hold() {
    let mut filter = DigitalInputFilter::default();
    let dt = 1.0 / 60.0;

    // A quick tap (4 frames = ~66 ms) on Balanced (140 ms pedal): light-to-medium braking
    let mut tap_brake = 0.0;
    for _ in 0..4 {
        let (_, _, b) = filter.update(0.0, 0.0, 1.0, dt);
        tap_brake = b;
    }
    println!("Quick 66ms tap brake value: {:.3}", tap_brake);
    assert!(
        (0.30..=0.55).contains(&tap_brake),
        "Quick tap must allow light-to-medium modulation (expected 0.30-0.55, got {tap_brake:.3})"
    );

    // Release key: brake resets to 0.0
    let (_, _, b_rel) = filter.update(0.0, 0.0, 0.0, dt);
    assert_eq!(b_rel, 0.0, "Releasing brake key must return to 0.0");

    // Sustained hold (15 frames = 250 ms) reaches full braking
    let mut hold_brake = 0.0;
    for _ in 0..15 {
        let (_, _, b) = filter.update(0.0, 0.0, 1.0, dt);
        hold_brake = b;
    }
    assert_eq!(hold_brake, 1.0, "Sustained brake key press must saturate at 1.0 full braking");

    // Raw pedals are instant
    let mut raw = DigitalInputFilter::new(SteeringProfile::Raw.to_config());
    let (_, _, b_raw) = raw.update(0.0, 0.0, 1.0, dt);
    assert_eq!(b_raw, 1.0);
}

/// Spec 043 successor of the double-attenuation regression test.
///
/// Steering is shaped exactly once: the input filter only handles timing, and the car maps full
/// input to its grip-aware authority. At 162 km/h full input must reach that authority (not 100%
/// mechanical lock, which is 3x past the front tire's useful angle), and the player's
/// Steering Authority (steer_overslip) must scale the wheel angle.
#[test]
fn test_player_car_physics_receives_unattenuated_steering_when_direct_or_raw() {
    use tdrace_core::Vec2;

    let dt = 1.0 / 60.0;
    let high_speed = 45.0; // 45 m/s ~ 162 km/h

    let steer_at = |overslip: f32| {
        let mut car = Car::new(CarConfig::sports_car());
        car.config.player = tdrace_core::physics::config::PlayerHandling::human(overslip, 0.0);
        car.state.velocity = Vec2::new(high_speed, 0.0);
        car.state.speed = high_speed;
        let authority = car.steer_authority(high_speed, 1.0);
        let ctrl = CarControls::new(0.0, 1.0, 0.0, false);
        // Rack reaches the target in a few frames; read it before the car slows or rotates much.
        for _ in 0..6 {
            car.step_per_wheel(&ctrl, [SurfaceType::Asphalt; 4], dt);
        }
        (car.state.steer_angle.abs(), authority, car.config.max_steer_angle)
    };

    let (angle_1, authority_1, lock) = steer_at(1.0);
    assert!(
        (angle_1 - authority_1).abs() < 0.02,
        "Full input must reach the grip-aware authority ({authority_1:.3}), got {angle_1:.3}"
    );
    assert!(authority_1 < lock * 0.5, "At 162 km/h the useful angle is far below mechanical lock");

    let (angle_13, _, _) = steer_at(1.3);
    assert!(
        angle_13 > angle_1 * 1.2,
        "Steering Authority 130% must steer clearly more than 100% ({angle_13:.3} vs {angle_1:.3})"
    );
}

