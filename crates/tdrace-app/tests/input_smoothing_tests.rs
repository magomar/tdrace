use tdrace_app::input::{DigitalInputConfig, DigitalInputFilter};
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;

#[test]
fn test_digital_input_filter_progressive_rise_and_centering() {
    let mut filter = DigitalInputFilter::default();
    let dt = 1.0 / 60.0;

    // First frame of holding steer right: should not snap instantly to 1.0
    let (s1, _, _) = filter.update(1.0, 0.0, 0.0, 0.0, dt);
    assert!(s1 > 0.0, "Steering should start moving right");
    assert!(s1 < 0.15, "First frame steering must be smoothed, was {s1}");

    // After 0.25s (15 frames at 60Hz), steering reaches full saturation
    for _ in 0..20 {
        filter.update(1.0, 0.0, 0.0, 0.0, dt);
    }
    let (s_sat, _, _) = filter.update(1.0, 0.0, 0.0, 0.0, dt);
    assert!((s_sat - 1.0).abs() < 1e-3, "Steering should saturate at 1.0, was {s_sat}");

    // Releasing key: centering rate should rapidly return towards 0.0
    let (s_rel, _, _) = filter.update(0.0, 0.0, 0.0, 0.0, dt);
    assert!(s_rel < 0.85, "Steering centering should be snappy on release, was {s_rel}");

    for _ in 0..10 {
        filter.update(0.0, 0.0, 0.0, 0.0, dt);
    }
    let (s_zero, _, _) = filter.update(0.0, 0.0, 0.0, 0.0, dt);
    assert!(s_zero.abs() < 0.05, "Steering should return near zero within 10 frames, was {s_zero}");
}

#[test]
fn test_speed_sensitive_steering_scaling() {
    let dt = 1.0 / 60.0;

    // Saturated steering at standstill (0 km/h) -> scale = 1.0
    let mut filter_0 = DigitalInputFilter::default();
    for _ in 0..25 {
        filter_0.update(1.0, 0.0, 0.0, 0.0, dt);
    }
    let (steer_0, _, _) = filter_0.update(1.0, 0.0, 0.0, 0.0, dt);
    assert_eq!(steer_0, 1.0, "Standstill steering must have full 1.0 lock");

    // Quick initial steer (6 frames at 60Hz = 100ms) at 30 m/s (~108 km/h)
    // is attenuated by base speed factor for stability
    let mut filter_quick = DigitalInputFilter::default();
    let mut quick_steer = 0.0;
    for _ in 0..6 {
        let (s, _, _) = filter_quick.update(1.0, 0.0, 0.0, 30.0, dt);
        quick_steer = s;
    }
    assert!(
        quick_steer < 0.75 && quick_steer > 0.40,
        "Initial high-speed turn response should be smoothed for stability, was {quick_steer}"
    );

    // Sustained key hold (30+ frames = >0.5s) bleeds off attenuation to 1.0 full lock
    let mut filter_hold = DigitalInputFilter::default();
    for _ in 0..30 {
        filter_hold.update(1.0, 0.0, 0.0, 30.0, dt);
    }
    let (steer_held, _, _) = filter_hold.update(1.0, 0.0, 0.0, 30.0, dt);
    assert!(
        (steer_held - 1.0).abs() < 1e-3,
        "Sustained key hold at speed must bleed to 1.0 full lock, was {steer_held}"
    );
}

#[test]
fn test_steering_profiles_configuration_and_cycling() {
    use tdrace_app::input::SteeringProfile;

    // 1. Balanced profile: recommended default
    let balanced_cfg = DigitalInputConfig::from_profile(SteeringProfile::Balanced);
    assert_eq!(balanced_cfg.profile, SteeringProfile::Balanced);
    assert!(balanced_cfg.speed_sensitive_enabled);
    assert_eq!(balanced_cfg.min_speed_steer_limit, 0.75);
    assert_eq!(balanced_cfg.hold_bleed_rate, 4.0);
    assert_eq!(balanced_cfg.steer_rise_rate, 8.0);

    // 2. Smooth profile: softer arcade
    let smooth_cfg = DigitalInputConfig::from_profile(SteeringProfile::Smooth);
    assert_eq!(smooth_cfg.profile, SteeringProfile::Smooth);
    assert!(smooth_cfg.speed_sensitive_enabled);
    assert_eq!(smooth_cfg.min_speed_steer_limit, 0.60);
    assert_eq!(smooth_cfg.hold_bleed_rate, 2.0);
    assert_eq!(smooth_cfg.steer_rise_rate, 6.0);

    // 3. Agile profile: sharp response for chicanes
    let agile_cfg = DigitalInputConfig::from_profile(SteeringProfile::Agile);
    assert_eq!(agile_cfg.profile, SteeringProfile::Agile);
    assert!(agile_cfg.speed_sensitive_enabled);
    assert_eq!(agile_cfg.min_speed_steer_limit, 0.80);
    assert_eq!(agile_cfg.hold_bleed_rate, 6.0);
    assert_eq!(agile_cfg.steer_rise_rate, 10.0);

    // 4. Direct profile: sim feel with speed attenuation switched off
    let direct_cfg = DigitalInputConfig::from_profile(SteeringProfile::Direct);
    assert_eq!(direct_cfg.profile, SteeringProfile::Direct);
    assert!(!direct_cfg.speed_sensitive_enabled);
    assert_eq!(direct_cfg.min_speed_steer_limit, 1.0);
    assert_eq!(direct_cfg.hold_bleed_rate, 8.0);
    assert_eq!(direct_cfg.steer_rise_rate, 12.0);

    // 5. Raw profile: esports zero-delay instant keys
    let raw_cfg = DigitalInputConfig::from_profile(SteeringProfile::Raw);
    assert_eq!(raw_cfg.profile, SteeringProfile::Raw);
    assert!(!raw_cfg.speed_sensitive_enabled);
    assert_eq!(raw_cfg.min_speed_steer_limit, 1.0);
    assert_eq!(raw_cfg.hold_bleed_rate, 10.0);
    assert_eq!(raw_cfg.steer_rise_rate, 20.0);

    // 5-profile Cycling: Balanced -> Smooth -> Agile -> Direct -> Raw -> Balanced
    assert_eq!(SteeringProfile::Balanced.cycle(), SteeringProfile::Smooth);
    assert_eq!(SteeringProfile::Smooth.cycle(), SteeringProfile::Agile);
    assert_eq!(SteeringProfile::Agile.cycle(), SteeringProfile::Direct);
    assert_eq!(SteeringProfile::Direct.cycle(), SteeringProfile::Raw);
    assert_eq!(SteeringProfile::Raw.cycle(), SteeringProfile::Balanced);

    // Index conversion
    for idx in 0..5 {
        let prof = SteeringProfile::from_index(idx);
        assert_eq!(prof.to_index(), idx);
    }
}

#[test]
fn test_speed_sensitive_switch_bypass() {
    let dt = 1.0 / 60.0;

    // Config with speed_sensitive_enabled = false
    let config = DigitalInputConfig::from_profile(tdrace_app::input::SteeringProfile::Direct);
    assert!(!config.speed_sensitive_enabled);
    let mut filter = DigitalInputFilter::new(config);

    // After saturation, turning at 30 m/s (~108 km/h) should yield 1.0 full steer immediately
    for _ in 0..20 {
        filter.update(1.0, 0.0, 0.0, 30.0, dt);
    }
    let (steer, _, _) = filter.update(1.0, 0.0, 0.0, 30.0, dt);
    assert_eq!(steer, 1.0, "Disabled speed sensitivity must allow 1.0 lock regardless of speed");

    // Re-enable speed sensitivity: attenuation applies at high speed
    filter.config.speed_sensitive_enabled = true;
    filter.config.speed_sensitive_factor = 0.005;
    filter.config.min_speed_steer_limit = 0.60;
    filter.config.hold_bleed_rate = 0.0; // Freeze bleed to verify base speed scaling
    filter.steer_hold_factor = 0.0;

    let (attenuated_steer, _, _) = filter.update(1.0, 0.0, 0.0, 30.0, dt);
    assert!(
        attenuated_steer < 0.95,
        "Enabled speed sensitivity must attenuate high speed steer, was {attenuated_steer}"
    );
}


#[test]
fn test_non_linear_center_micro_corrections() {
    let config = DigitalInputConfig {
        steer_exponent: 1.4,
        speed_sensitive_factor: 0.0,
        ..Default::default()
    };
    let mut filter = DigitalInputFilter::new(config);
    filter.current_steer = 0.3; // 30% digital input

    let (steer, _, _) = filter.update(0.3, 0.0, 0.0, 0.0, 1.0 / 60.0);
    // 0.3^1.4 = ~0.184 (soft center response)
    assert!(
        steer < 0.22 && steer > 0.15,
        "Non-linear gamma curve should soften 30% steer down to ~18%, was {steer}"
    );
}

#[test]
fn test_vehicle_high_speed_turn_stability_with_smoothed_input() {
    let dt = 1.0 / 60.0;
    let mut car = Car::new(CarConfig::sports_car());
    let mut filter = DigitalInputFilter::default();

    // 1. Accelerate in a straight line (240 frames = 4 seconds)
    for _ in 0..240 {
        let (_, throttle, _) = filter.update(0.0, 1.0, 0.0, car.state.speed, dt);
        let ctrl = CarControls::new(throttle, 0.0, 0.0, false);
        car.step(&ctrl, SurfaceType::Asphalt, dt);
    }
    assert!(car.speed_kmh() > 30.0, "Car must reach speed, was {:.1} km/h", car.speed_kmh());

    // 2. Perform lane change / gentle turn by holding steer right for 20 frames
    for _ in 0..20 {
        let (steer, throttle, _) = filter.update(1.0, 0.8, 0.0, car.state.speed, dt);
        let ctrl = CarControls::new(throttle, steer, 0.0, false);
        car.step(&ctrl, SurfaceType::Asphalt, dt);
    }

    // 3. Center steering for 20 frames
    for _ in 0..20 {
        let (steer, throttle, _) = filter.update(0.0, 0.8, 0.0, car.state.speed, dt);
        let ctrl = CarControls::new(throttle, steer, 0.0, false);
        car.step(&ctrl, SurfaceType::Asphalt, dt);
    }

    // Vehicle should NOT have spun out
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

    // A quick tap (4 frames = ~66ms): should produce gentle/medium progressive braking, not instant 1.0 lock
    let mut tap_brake = 0.0;
    for _ in 0..4 {
        let (_, _, b) = filter.update(0.0, 0.0, 1.0, 20.0, dt);
        tap_brake = b;
    }
    println!("Quick 66ms tap brake value: {:.3}", tap_brake);
    assert!(
        tap_brake >= 0.30 && tap_brake <= 0.55,
        "Quick tap must allow light-to-medium modulation (expected 0.30-0.55, got {tap_brake:.3})"
    );

    // Release key: brake resets to 0.0
    let (_, _, b_rel) = filter.update(0.0, 0.0, 0.0, 20.0, dt);
    assert_eq!(b_rel, 0.0, "Releasing brake key must return to 0.0");

    // Sustained hold (15 frames = 250ms): smoothly reaches full 1.0 saturation
    let mut hold_brake = 0.0;
    for _ in 0..15 {
        let (_, _, b) = filter.update(0.0, 0.0, 1.0, 20.0, dt);
        hold_brake = b;
    }
    println!("Sustained 250ms hold brake value: {:.3}", hold_brake);
    assert_eq!(hold_brake, 1.0, "Sustained brake key press must saturate at 1.0 full braking");
}

#[test]
fn test_interactive_controls_filter_sync_and_persistence() {
    use tdrace_app::input::SteeringProfile;

    // 1. Verify profile-to-config presets
    let bal = SteeringProfile::Balanced.to_config();
    assert_eq!(bal.profile, SteeringProfile::Balanced);
    assert!((bal.hold_bleed_rate - 4.0).abs() < 1e-3);
    assert!((bal.min_speed_steer_limit - 0.75).abs() < 1e-3);

    let smo = SteeringProfile::Smooth.to_config();
    assert_eq!(smo.profile, SteeringProfile::Smooth);
    assert!((smo.hold_bleed_rate - 2.0).abs() < 1e-3);
    assert!((smo.min_speed_steer_limit - 0.60).abs() < 1e-3);

    let agi = SteeringProfile::Agile.to_config();
    assert_eq!(agi.profile, SteeringProfile::Agile);
    assert!((agi.hold_bleed_rate - 6.0).abs() < 1e-3);
    assert!((agi.min_speed_steer_limit - 0.80).abs() < 1e-3);

    let dir = SteeringProfile::Direct.to_config();
    assert_eq!(dir.profile, SteeringProfile::Direct);
    assert!((dir.hold_bleed_rate - 8.0).abs() < 1e-3);
    assert!((dir.min_speed_steer_limit - 1.00).abs() < 1e-3);

    let raw = SteeringProfile::Raw.to_config();
    assert_eq!(raw.profile, SteeringProfile::Raw);
    assert!((raw.hold_bleed_rate - 10.0).abs() < 1e-3);
    assert!((raw.min_speed_steer_limit - 1.00).abs() < 1e-3);

    // 2. Index round-trips
    assert_eq!(SteeringProfile::from_index(0), SteeringProfile::Balanced);
    assert_eq!(SteeringProfile::from_index(1), SteeringProfile::Smooth);
    assert_eq!(SteeringProfile::from_index(2), SteeringProfile::Agile);
    assert_eq!(SteeringProfile::from_index(3), SteeringProfile::Direct);
    assert_eq!(SteeringProfile::from_index(4), SteeringProfile::Raw);
    assert_eq!(SteeringProfile::from_index(99), SteeringProfile::Balanced);
    assert_eq!(SteeringProfile::Balanced.to_index(), 0);
    assert_eq!(SteeringProfile::Smooth.to_index(), 1);
    assert_eq!(SteeringProfile::Agile.to_index(), 2);
    assert_eq!(SteeringProfile::Direct.to_index(), 3);
    assert_eq!(SteeringProfile::Raw.to_index(), 4);

    // 3. Bleed rate dynamic behavior: higher bleed rate recovers full lock faster
    let dt = 1.0 / 60.0;
    let mut slow_filter = DigitalInputFilter::new(DigitalInputConfig {
        hold_bleed_rate: 1.0,
        min_speed_steer_limit: 0.50,
        ..bal
    });
    let mut fast_filter = DigitalInputFilter::new(DigitalInputConfig {
        hold_bleed_rate: 4.0,
        min_speed_steer_limit: 0.50,
        ..bal
    });

    let mut slow_steer = 0.0;
    let mut fast_steer = 0.0;
    for _ in 0..10 {
        let (s, _, _) = slow_filter.update(1.0, 0.0, 0.0, 25.0, dt);
        slow_steer = s;
        let (f, _, _) = fast_filter.update(1.0, 0.0, 0.0, 25.0, dt);
        fast_steer = f;
    }

    assert!(
        fast_steer > slow_steer,
        "Faster bleed rate ({fast_steer}) should restore more steering angle than slow bleed rate ({slow_steer})"
    );
}

#[test]
fn test_player_car_physics_receives_unattenuated_steering_when_direct_or_raw() {
    use tdrace_app::input::SteeringProfile;
    use tdrace_core::Vec2;

    let dt = 1.0 / 60.0;
    let high_speed = 45.0; // 45 m/s ~ 162 km/h

    // 1. Direct/Raw profile filter
    let direct_cfg = SteeringProfile::Direct.to_config();
    assert!(!direct_cfg.speed_sensitive_enabled);
    assert_eq!(direct_cfg.min_speed_steer_limit, 1.0);
    let mut filter = DigitalInputFilter::new(direct_cfg);

    // After ramp, filter outputs 1.0 full lock at 162 km/h
    for _ in 0..15 {
        filter.update(1.0, 0.0, 0.0, high_speed, dt);
    }
    let (filter_steer, _, _) = filter.update(1.0, 0.0, 0.0, high_speed, dt);
    assert_eq!(filter_steer, 1.0);

    // 2. Player car with speed_sensitive_steer_factor = 0.0 (applied by RaceSession)
    let mut player_car = Car::new(CarConfig::sports_car());
    player_car.config.speed_sensitive_steer_factor = 0.0;
    player_car.state.velocity = Vec2::new(high_speed, 0.0);
    player_car.state.speed = high_speed;

    let ctrl = CarControls::new(0.0, filter_steer, 0.0, false);
    for _ in 0..30 {
        player_car.step_per_wheel(&ctrl, [SurfaceType::Asphalt; 4], dt);
    }

    // Player car must reach 100% of max_steer_angle at high speed without physics attenuation
    let max_steer = player_car.config.max_steer_angle;
    assert!(
        (player_car.state.steer_angle.abs() - max_steer).abs() < 1e-2,
        "Player car steering angle ({}) must reach full mechanical lock ({}) at high speed when Direct/Raw",
        player_car.state.steer_angle.abs(),
        max_steer
    );

    // 3. Contrast with unconfigured car (legacy double-attenuation with speed_sensitive_steer_factor > 0)
    let mut legacy_car = Car::new(CarConfig::sports_car());
    legacy_car.config.speed_sensitive_steer_factor = 0.016; // Legacy GT factor
    legacy_car.state.velocity = Vec2::new(high_speed, 0.0);
    legacy_car.state.speed = high_speed;

    for _ in 0..30 {
        legacy_car.step_per_wheel(&ctrl, [SurfaceType::Asphalt; 4], dt);
    }
    let legacy_steer = legacy_car.state.steer_angle.abs();
    assert!(
        legacy_steer < max_steer * 0.65,
        "Legacy car was heavily attenuated to only {} of max lock",
        legacy_steer / max_steer
    );
    assert!(
        player_car.state.steer_angle.abs() > legacy_steer * 1.5,
        "Player car with settings applied must have significantly greater steering angle than legacy attenuated car"
    );
}

