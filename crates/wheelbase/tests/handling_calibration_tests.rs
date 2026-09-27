//! Handling calibration gates (Spec 042).
//!
//! Each test measures vehicle *behavior* (yaw, sideslip, axle saturation, distances), not
//! internal parameters, so tuning can change freely as long as the feel targets hold.

use wheelbase::{Car, CarConfig, CarControls, DriverAssistsConfig, PlayerHandling, SurfaceType, Vec2};

const DT: f32 = 1.0 / 120.0;

/// Mean normalized slip |alpha| / alpha_peak of an axle over the last second of a steady corner.
fn steady_axle_slip(roll_balance: f32) -> (f32, f32) {
    let mut cfg = CarConfig::sports_car();
    cfg.roll_balance = roll_balance;
    cfg.assists = DriverAssistsConfig::raw();
    let mut car = Car::new(cfg);
    car.set_velocity(Vec2::new(25.0, 0.0));
    let (mut front, mut rear) = (0.0f32, 0.0f32);
    for step in 0..480 {
        let throttle = ((25.0 - car.state.speed) * 0.3).clamp(0.0, 0.6);
        car.step(&CarControls::new(throttle, 0.12, 0.0, false), SurfaceType::Asphalt, DT);
        if step >= 360 {
            let w = &car.state.wheels;
            front += (w[0].slip_angle.abs() + w[1].slip_angle.abs()) / 240.0;
            rear += (w[2].slip_angle.abs() + w[3].slip_angle.abs()) / 240.0;
        }
    }
    let peak = car.config.tire.peak_slip_angle();
    (front / peak, rear / peak)
}

/// Which axle runs the larger normalized slip (the one closer to / past its limit).
fn first_axle_to_saturate(roll_balance: f32) -> &'static str {
    let (front, rear) = steady_axle_slip(roll_balance);
    println!("roll_balance {roll_balance:.2}: front {front:.2} x peak, rear {rear:.2} x peak");
    if rear > front { "rear" } else { "front" }
}

/// Scenario: Roll balance flips the handling balance
///
/// Given roll_balance 0.40 and then 0.60
/// When the car holds a steady corner at 25 m/s
/// Then the rear axle saturates first at 0.40 and the front axle saturates first at 0.60
#[test]
fn test_roll_balance_flips_which_axle_saturates_first() {
    let loose = first_axle_to_saturate(0.40);
    let tight = first_axle_to_saturate(0.60);
    println!("roll_balance 0.40 -> {loose} first, 0.60 -> {tight} first");
    assert_eq!(loose, "rear");
    assert_eq!(tight, "front");
}

/// Steady path curvature yaw/speed (1/m, mean of the last 0.5 s of a 2 s hold) with throttle
/// holding `speed`, plus the peak sideslip. Curvature, not yaw rate: when the front plows the car
/// also slows, and yaw rate would fall with speed even though the turn did not open up.
fn steady_yaw(cfg: &CarConfig, speed: f32, steer: f32) -> (f32, f32) {
    let mut car = Car::new(cfg.clone());
    car.set_velocity(Vec2::new(speed, 0.0));
    let (mut curvature, mut beta) = (0.0f32, 0.0f32);
    for step in 0..240 {
        let throttle = ((speed - car.state.speed) * 0.5).clamp(0.0, 1.0);
        car.step(&CarControls::new(throttle, steer, 0.0, false), SurfaceType::Asphalt, DT);
        beta = beta.max(car.state.sideslip_angle.abs());
        if step >= 180 {
            curvature += car.state.angular_velocity.abs() / car.state.speed.max(1.0) / 60.0;
        }
    }
    (curvature, beta)
}

/// Scenario: Steering response is monotonic at every speed
///
/// Given overslip settings 0.85 to 1.30 and speeds of 10, 25 and 45 m/s, throttle holding speed
/// When a held steer input sweeps 0.1 -> 1.0
/// Then the steady turn (path curvature) never opens up by more than 3% as input grows
#[test]
fn test_steering_response_is_monotonic_at_every_speed() {
    for overslip in [0.85f32, 1.0, 1.15, 1.30] {
        let mut cfg = CarConfig::sports_car();
        cfg.player = PlayerHandling::human(overslip, 0.0);
        for speed in [10.0f32, 25.0, 45.0] {
            let mut prev = 0.0f32;
            let mut row = String::new();
            for i in 1..=10 {
                let steer = i as f32 / 10.0;
                let (yaw, beta) = steady_yaw(&cfg, speed, steer);
                row.push_str(&format!(" {steer:.1}:{yaw:.4}/{beta:.2}"));
                assert!(
                    yaw >= prev * 0.97,
                    "overslip {overslip} at {speed} m/s: curvature dropped from {prev:.4} to {yaw:.4} at steer {steer:.1}\n{row}"
                );
                prev = prev.max(yaw);
            }
            println!("overslip {overslip:.2} v={speed:>4.1}:{row}");
        }
    }
}
