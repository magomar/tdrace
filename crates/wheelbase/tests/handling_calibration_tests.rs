//! Handling calibration gates (Spec 043).
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
/// Given the preset authorities (Smooth 0.90, Balanced 1.00, Sharp 1.07, Raw 1.15) and speeds of
/// 10, 25 and 45 m/s, throttle holding speed
/// When a held steer input sweeps 0.1 -> 1.0
/// Then the steady turn (path curvature) never opens up by more than 3% as input grows
#[test]
fn test_steering_response_is_monotonic_at_every_speed() {
    for overslip in [0.90f32, 1.0, 1.07, 1.15] {
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

/// Scenario: Corner exit with W held keeps drive
///
/// Given 20 m/s, steer 0.4, W held for 2 s, arcade assists, Balanced handling
/// When the car exits the corner
/// Then traction control keeps >= 50% of the requested drive force on average and the exit
/// speed is >= 20.7 m/s
///
/// Spec 043 note: the draft gate counted TCS-active frames (<= 50%). This car asks for 6.8 kN at
/// 20 m/s while its rear tires can transmit ~5 kN, so a correct TCS trims torque on every frame;
/// the frame count measured "TCS present", not "TCS strangling". The delivered share measures the
/// real complaint (pre-043: TCS cut deeply and exit speed fell 12%).
#[test]
fn test_corner_exit_with_throttle_held_keeps_drive() {
    // TCS alone: traction help (a separate player aid) off.
    let mut cfg = CarConfig::sports_car();
    cfg.player = PlayerHandling::human(1.0, 0.0);
    let requested = cfg.max_engine_force;
    let mut car = Car::new(cfg);
    car.set_velocity(Vec2::new(20.0, 0.0));
    let mut delivered = 0.0f32;
    for _ in 0..240 {
        car.step(&CarControls::new(1.0, 0.4, 0.0, false), SurfaceType::Asphalt, DT);
        let w = &car.state.wheels;
        delivered += (w[2].longitudinal_force + w[3].longitudinal_force).max(0.0) / 240.0;
    }
    let share = delivered / requested;
    println!("corner exit: delivered {:.0} N of {requested:.0} N ({:.0}%), exit speed {:.2} m/s", delivered, share * 100.0, car.state.speed);
    assert!(share >= 0.5, "TCS / traction help delivered only {:.0}% of the requested drive", share * 100.0);
    assert!(car.state.speed >= 20.7, "exit speed {:.2} m/s", car.state.speed);
}

/// Scenario: Lift-off is progressive
///
/// Given 40 m/s, steer 0.25, throttle for 1 s and then released, Balanced handling
/// When the car coasts for 1.5 s
/// Then the peak sideslip is <= 0.705 rad
#[test]
fn test_lift_off_is_progressive() {
    let mut cfg = CarConfig::sports_car();
    cfg.player = PlayerHandling::human(1.0, 0.5);
    let mut car = Car::new(cfg);
    car.set_velocity(Vec2::new(40.0, 0.0));
    let mut peak_beta = 0.0f32;
    for step in 0..300 {
        let throttle = if step < 120 { 1.0 } else { 0.0 };
        car.step(&CarControls::new(throttle, 0.25, 0.0, false), SurfaceType::Asphalt, DT);
        if step >= 120 {
            peak_beta = peak_beta.max(car.state.sideslip_angle.abs());
        }
    }
    println!("lift-off peak sideslip {peak_beta:.3} rad");
    assert!(peak_beta <= 0.705, "lift-off peak sideslip {peak_beta:.3} rad");
}

fn braking_distance(steer: f32) -> f32 {
    // A keyboard driver: Balanced handling (grip-aware steering).
    let mut cfg = CarConfig::sports_car();
    cfg.player = PlayerHandling::human(1.0, 0.5);
    let mut car = Car::new(cfg);
    car.set_velocity(Vec2::new(40.0, 0.0));
    let start = car.state.position;
    let mut t = 0.0;
    while car.state.speed > 10.0 && t < 10.0 {
        car.step(&CarControls::new(0.0, steer, 1.0, false), SurfaceType::Asphalt, DT);
        t += DT;
    }
    (car.state.position - start).length()
}

/// Scenario: A small steer input does not weaken the brakes
///
/// Given braking from 40 to 10 m/s
/// When steer is 0.00 and then 0.05
/// Then the stopping distances differ by < 3%
#[test]
fn test_small_steer_does_not_weaken_brakes() {
    let straight = braking_distance(0.0);
    let steered = braking_distance(0.05);
    println!("braking 40->10 m/s: straight {straight:.1} m, steer 0.05 {steered:.1} m");
    assert!((steered - straight).abs() / straight < 0.03);
}

fn skidpad_lateral_g(grip: f32) -> f32 {
    // Closed-loop constant-radius skidpad (protocol C): the car's cornering capability.
    let mut cfg = CarConfig::sports_car();
    cfg.tire.grip = grip;
    wheelbase::sim::protocols::run_protocol_c(&cfg, SurfaceType::Asphalt, 30.0, DT).peak_lateral_accel_g
}

/// Scenario: Car tuning knobs reach the tires
///
/// Given `tire.grip` 1.0 and then 1.3
/// When the car drives a 30 m skidpad at its limit
/// Then the lateral g rises by >= 25%
#[test]
fn test_grip_knob_moves_lateral_g() {
    let base = skidpad_lateral_g(1.0);
    let grippy = skidpad_lateral_g(1.3);
    println!("skidpad lateral g: grip 1.0 -> {base:.3} g, grip 1.3 -> {grippy:.3} g");
    assert!(grippy >= base * 1.25, "grip 1.3 must raise lateral g by 25% ({base:.3} -> {grippy:.3})");
}

/// Scenario: The model is numerically stable
///
/// Given every factory CarConfig preset
/// When 60 s of random inputs run from 0 to 70 m/s, including full throttle, full brake,
/// handbrake and reverse at standstill
/// Then no state becomes non-finite and |omega_wheel| stays <= 550 rad/s
#[test]
fn test_random_input_fuzz_is_numerically_stable() {
    let presets = [
        ("sports", CarConfig::sports_car()),
        ("drift", CarConfig::drift_car()),
        ("kart", CarConfig::kart()),
        ("rally", CarConfig::rally_car()),
        ("stock", CarConfig::stock_car_ta1()),
        ("sand", CarConfig::sand_rail()),
    ];
    let surfaces = [SurfaceType::Asphalt, SurfaceType::Gravel, SurfaceType::Grass, SurfaceType::SheetIce, SurfaceType::DeepSand];
    for (name, cfg) in presets {
        for human in [false, true] {
            let mut cfg = cfg.clone();
            if human {
                cfg.player = PlayerHandling::human(1.3, 0.0);
            }
            let mut car = Car::new(cfg);
            let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
            let mut rng = move || {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                ((state >> 33) as f32) / (u32::MAX >> 1) as f32
            };
            let mut ctrl = CarControls::default();
            let mut surf = SurfaceType::Asphalt;
            for step in 0..(60 * 120) {
                if step % 30 == 0 {
                    ctrl = CarControls {
                        throttle: if rng() > 0.3 { 1.0 } else { rng() },
                        steer: rng() * 2.0 - 1.0,
                        brake: if rng() > 0.8 { rng() } else { 0.0 },
                        handbrake: rng() > 0.9,
                        reverse: car.state.speed < 1.0 && rng() > 0.8,
                    };
                    surf = surfaces[(rng() * surfaces.len() as f32) as usize % surfaces.len()];
                }
                car.step(&ctrl, surf, DT);
                let s = &car.state;
                assert!(
                    s.position.is_finite() && s.velocity.is_finite() && s.angular_velocity.is_finite() && s.angle.is_finite(),
                    "{name} (human={human}) non-finite state at step {step}"
                );
                for w in &s.wheel_assemblies {
                    assert!(w.angular_velocity.is_finite() && w.angular_velocity.abs() <= 550.0, "{name} wheel omega {}", w.angular_velocity);
                }
                assert!(s.speed < 120.0, "{name} (human={human}) runaway speed {:.1} at step {step}", s.speed);
            }
        }
    }
}

/// Seconds from a standstill to 25 m/s in a straight line with W held.
fn launch_time(traction_help: f32) -> f32 {
    let mut cfg = CarConfig::sports_car();
    cfg.player = PlayerHandling::human(1.0, traction_help);
    let mut car = Car::new(cfg);
    for step in 0..(20 * 120) {
        car.step(&CarControls::new(1.0, 0.0, 0.0, false), SurfaceType::Asphalt, DT);
        if car.state.speed >= 25.0 {
            return step as f32 * DT;
        }
    }
    f32::INFINITY
}

/// Scenario: Traction help does not ease straight-line drive
///
/// Given the sports car at a standstill
/// When W is held on a straight with traction help 0, 0.7 and 0.9
/// Then the time to 25 m/s with help is within 2% of the time without it
///
/// Pre-fix: traction help counted the rear tires' drive force as "near the limit", so it cut the
/// throttle on every launch and straight (a 92.8 s standing-start lap at 0.9) (tdrace-izlq).
#[test]
fn test_traction_help_does_not_ease_straight_line_drive() {
    let base = launch_time(0.0);
    for th in [0.7, 0.9] {
        let t = launch_time(th);
        println!("0-25 m/s: traction help 0 {base:.2} s, {th} {t:.2} s");
        assert!(t <= base * 1.02, "traction help {th} slowed the launch: {t:.2} s vs {base:.2} s");
    }
}

/// Scenario: Traction help still catches power oversteer
///
/// Given the sports car with all assists off at 20 m/s
/// When full steer and W are held for 2 s
/// Then it spins without traction help (peak sideslip > 1 rad) and not with 0.7 (< 0.15 rad)
#[test]
fn test_traction_help_catches_power_oversteer() {
    let peak_sideslip = |traction_help: f32| {
        let mut cfg = CarConfig::sports_car();
        cfg.assists = DriverAssistsConfig::raw();
        cfg.player = PlayerHandling::human(1.0, traction_help);
        let mut car = Car::new(cfg);
        car.set_velocity(Vec2::new(20.0, 0.0));
        let mut peak = 0.0f32;
        for _ in 0..240 {
            car.step(&CarControls::new(1.0, 1.0, 0.0, false), SurfaceType::Asphalt, DT);
            peak = peak.max(car.state.sideslip_angle.abs());
        }
        peak
    };
    let (off, on) = (peak_sideslip(0.0), peak_sideslip(0.7));
    println!("power oversteer @20 m/s: peak sideslip {off:.3} rad without help, {on:.3} rad with 0.7");
    assert!(off > 1.0, "the scenario must spin without help ({off:.3} rad)");
    assert!(on < 0.15, "traction help 0.7 did not catch the slide ({on:.3} rad)");
}
