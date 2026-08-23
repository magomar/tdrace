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
    let mut filter = DigitalInputFilter::default();
    let dt = 1.0 / 60.0;

    // Saturated steering at standstill (0 km/h) -> scale = 1.0
    for _ in 0..40 {
        filter.update(1.0, 0.0, 0.0, 0.0, dt);
    }
    let (steer_0, _, _) = filter.update(1.0, 0.0, 0.0, 0.0, dt);
    assert_eq!(steer_0, 1.0, "Standstill steering must have full 1.0 lock");

    // Saturated steering at 30 m/s (~108 km/h) -> scale is responsive (~0.87)
    let mut filter_med = DigitalInputFilter::default();
    for _ in 0..40 {
        filter_med.update(1.0, 0.0, 0.0, 30.0, dt);
    }
    let (steer_med, _, _) = filter_med.update(1.0, 0.0, 0.0, 30.0, dt);
    assert!(steer_med < 0.95 && steer_med > 0.80, "Medium speed steer was {steer_med}");

    // Saturated steering at 60 m/s (~216 km/h) -> scale maintains high turning authority (~0.77)
    let mut filter_high = DigitalInputFilter::default();
    for _ in 0..40 {
        filter_high.update(1.0, 0.0, 0.0, 60.0, dt);
    }
    let (steer_high, _, _) = filter_high.update(1.0, 0.0, 0.0, 60.0, dt);
    assert!(steer_high < 0.85 && steer_high >= 0.65, "High speed steer was {steer_high}");
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
