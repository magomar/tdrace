use wheelbase::car::{Car, CarControls};
use wheelbase::config::{CarConfig, DifferentialType, DriverAssistsConfig};
use wheelbase::surface::SurfaceType;

/// Scenario: Spool differential locks wheel rotational velocities identically
///
/// Given a vehicle equipped with a Spool (solid / locked) rear differential
/// When driven through dynamic cornering and throttle applications
/// Then both rear wheel assemblies must maintain identical angular velocities across all timesteps
#[test]
fn test_spool_differential_locks_wheel_rotational_velocities() {
    let mut cfg = CarConfig::kart();
    cfg.rear_differential = DifferentialType::Spool;
    let mut car = Car::new(cfg);
    let dt = 1.0 / 60.0;

    // Apply forward throttle and aggressive steering
    let ctrl = CarControls::new(1.0, 0.8, 0.0, false);
    for step in 0..180 {
        car.step(&ctrl, SurfaceType::Asphalt, dt);

        let omega_rl = car.state.wheel_assemblies[2].angular_velocity;
        let omega_rr = car.state.wheel_assemblies[3].angular_velocity;

        assert!(
            (omega_rl - omega_rr).abs() < 1e-4,
            "Step {}: Spool differential must strictly lock rear wheel angular velocities (RL={}, RR={})",
            step,
            omega_rl,
            omega_rr
        );
    }
}

/// Scenario: LSD transfers drive torque to gripping wheel on split-mu surface
///
/// Given two identical sports cars: one with an Open differential and one with a Salisbury LSD
/// When launched from a standstill on a split-mu surface (left wheels on SheetIce, right wheels on Asphalt)
/// Then the LSD vehicle must transfer drive torque to the high-traction right wheel
/// And the LSD vehicle must achieve at least 30% higher forward velocity than the Open differential vehicle
#[test]
fn test_lsd_transfers_torque_to_gripping_wheel_proportional_to_locking_factor() {
    let dt = 1.0 / 60.0;

    // 1. Vehicle with Open differential
    let mut cfg_open = CarConfig::sports_car();
    cfg_open.rear_differential = DifferentialType::Open;
    cfg_open.front_differential = DifferentialType::Open;
    let mut car_open = Car::new(cfg_open);

    // 2. Vehicle with Limited Slip Differential
    let mut cfg_lsd = CarConfig::sports_car();
    cfg_lsd.rear_differential = DifferentialType::LimitedSlip {
        power_lock: 0.50,
        coast_lock: 0.25,
        preload_nm: 60.0,
    };
    let mut car_lsd = Car::new(cfg_lsd);

    let ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    let split_surfaces = [
        SurfaceType::SheetIce, // FL
        SurfaceType::Asphalt,  // FR
        SurfaceType::SheetIce, // RL
        SurfaceType::Asphalt,  // RR
    ];

    for _ in 0..120 {
        car_open.step_per_wheel(&ctrl, split_surfaces, dt);
        car_lsd.step_per_wheel(&ctrl, split_surfaces, dt);
    }

    let speed_open = car_open.state.local_velocity.x;
    let speed_lsd = car_lsd.state.local_velocity.x;

    println!(
        "Split-mu Acceleration: Open Diff Speed = {:.2} m/s ({:.1} km/h), LSD Speed = {:.2} m/s ({:.1} km/h)",
        speed_open,
        speed_open * 3.6,
        speed_lsd,
        speed_lsd * 3.6
    );

    assert!(
        speed_lsd > speed_open * 1.30,
        "LSD vehicle must achieve at least 30% higher speed on split-mu than Open diff (got Open={:.2}, LSD={:.2})",
        speed_open,
        speed_lsd
    );
}

/// Scenario: Spool differential delivers 100% drive torque to gripping wheel on split-mu surface
///
/// Given a race vehicle equipped with a Spool rear differential
/// When launched on a split-mu surface where one driven wheel is on SheetIce and one is on Asphalt
/// Then the Spool differential must deliver full drive thrust to the gripping wheel without one-wheel runaway
/// And achieve higher forward acceleration than an Open differential
///
/// Spec 042: both cars run full (arcade) traction control. An open diff under engine-torque TCS is
/// held to twice the ice wheel's grip, while the spool passes torque to the asphalt side. With the
/// stock car's mild sport TCS (0.40) half the axle torque already saturates the asphalt tire in both
/// cases, so the two diffs are grip-limited alike (measured 4.69 vs 4.66 m/s): physically correct.
#[test]
fn test_spool_differential_transfers_100_percent_torque_when_one_wheel_unloaded() {
    let dt = 1.0 / 60.0;

    let mut cfg_spool = CarConfig::stock_car_ta1();
    cfg_spool.rear_differential = DifferentialType::Spool;
    cfg_spool.assists = DriverAssistsConfig::arcade();
    let mut car_spool = Car::new(cfg_spool);

    let mut cfg_open = CarConfig::stock_car_ta1();
    cfg_open.rear_differential = DifferentialType::Open;
    cfg_open.assists = DriverAssistsConfig::arcade();
    let mut car_open = Car::new(cfg_open);

    let ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    let split_surfaces = [
        SurfaceType::SheetIce,
        SurfaceType::Asphalt,
        SurfaceType::SheetIce,
        SurfaceType::Asphalt,
    ];

    for _ in 0..120 {
        car_spool.step_per_wheel(&ctrl, split_surfaces, dt);
        car_open.step_per_wheel(&ctrl, split_surfaces, dt);
    }

    let speed_spool = car_spool.state.local_velocity.x;
    let speed_open = car_open.state.local_velocity.x;

    println!(
        "Spool vs Open Split-mu: Spool Speed = {:.2} m/s, Open Speed = {:.2} m/s",
        speed_spool, speed_open
    );

    assert!(
        speed_spool > speed_open * 1.40,
        "Spool differential must achieve >= 40% higher speed than Open diff on split-mu (Spool={:.2}, Open={:.2})",
        speed_spool,
        speed_open
    );
}

/// Scenario: Open differential equalizes drive torque 50/50 and limits traction on split-mu
///
/// Given a vehicle equipped with an Open rear differential
/// When one driven wheel loses grip on a zero-friction ice patch
/// Then drive torque on both wheels is capped by the low-grip wheel
/// Resulting in inside wheel spin and limited forward thrust
#[test]
fn test_open_differential_equalizes_torque_and_limits_power_on_split_mu() {
    let dt = 1.0 / 60.0;
    let mut cfg_open = CarConfig::sports_car();
    cfg_open.rear_differential = DifferentialType::Open;
    let mut car_open = Car::new(cfg_open);

    let ctrl = CarControls::new(0.8, 0.0, 0.0, false);
    let split_surfaces = [
        SurfaceType::SheetIce,
        SurfaceType::Asphalt,
        SurfaceType::SheetIce,
        SurfaceType::Asphalt,
    ];

    for _ in 0..60 {
        car_open.step_per_wheel(&ctrl, split_surfaces, dt);
    }

    let w_ice = car_open.state.wheel_assemblies[2].angular_velocity;
    let w_asphalt = car_open.state.wheel_assemblies[3].angular_velocity;

    println!(
        "Open Diff Slip: Wheel on Ice Omega = {:.2} rad/s, Wheel on Asphalt Omega = {:.2} rad/s",
        w_ice, w_asphalt
    );

    // Open diff allows the low-traction wheel to spin rapidly while the gripping wheel has much lower speed
    assert!(
        w_ice > w_asphalt * 1.5,
        "Open diff must allow the low-friction wheel to spin up while the gripping wheel lags (Ice={:.2}, Asphalt={:.2})",
        w_ice,
        w_asphalt
    );
}

/// Scenario: Kart spool differential with caster jacking maintains tight turning circle and drive
///
/// Given a competition go-kart with Spool rear axle and caster jacking
/// When executing maximum steer at low speed
/// Then the turning diameter must measure less than 2.60 meters
/// And the rear axle angular velocities remain locked without spin runaway
#[test]
fn test_kart_spool_with_caster_jacking_maintains_drive_and_turning_circle() {
    let cfg = CarConfig::kart();
    assert_eq!(cfg.rear_differential, DifferentialType::Spool);

    let mut car = Car::new(cfg);
    let dt = 1.0 / 60.0;

    // Settle into steady-state circle at low speed
    let ctrl = CarControls::new(0.20, 1.0, 0.0, false);
    for _ in 0..120 {
        car.step(&ctrl, SurfaceType::Asphalt, dt);
    }

    let speed = car.state.local_velocity.x.abs();
    let omega = car.state.angular_velocity.abs();
    let radius = speed / omega;
    let diameter = radius * 2.0;

    println!(
        "Kart Spool Turning: Speed = {:.2} m/s, Yaw Rate = {:.2} rad/s, Turning Diameter = {:.2} m",
        speed, omega, diameter
    );

    assert!(
        diameter < 2.60,
        "Kart with Spool axle must maintain turning diameter < 2.60m (got {:.2}m)",
        diameter
    );

    // Verify rear wheels remain synchronized
    let omega_rl = car.state.wheel_assemblies[2].angular_velocity;
    let omega_rr = car.state.wheel_assemblies[3].angular_velocity;
    assert!(
        (omega_rl - omega_rr).abs() < 1e-4,
        "Kart rear wheels must remain locked on solid spool axle (RL={}, RR={})",
        omega_rl,
        omega_rr
    );
}

fn steady_yaw_rate(diff: DifferentialType, throttle: f32, steer: f32, v0: f32) -> f32 {
    let mut cfg = CarConfig::sports_car();
    cfg.rear_differential = diff;
    cfg.assists = DriverAssistsConfig::raw();
    let mut car = Car::new(cfg);
    car.set_velocity(wheelbase::Vec2::new(v0, 0.0));
    let ctrl = CarControls::new(throttle, steer, 0.0, false);
    let mut yaw = 0.0;
    for step in 0..240 {
        car.step(&ctrl, SurfaceType::Asphalt, 1.0 / 120.0);
        if step >= 180 {
            yaw += car.state.angular_velocity.abs() / 60.0;
        }
    }
    yaw
}

/// Scenario: A locked axle resists turning (Spec 042)
///
/// Given the same car with a Spool and with an Open rear differential
/// When it holds the same moderate steer input at 15 m/s
/// Then the Spool car turns with a lower steady yaw rate (the inner wheel is dragged forward and
/// the outer wheel held back: an understeer moment, not the pre-042 torque-vectoring sign)
#[test]
fn test_spool_produces_understeer_moment_versus_open() {
    let yaw_open = steady_yaw_rate(DifferentialType::Open, 0.3, 0.15, 15.0);
    let yaw_spool = steady_yaw_rate(DifferentialType::Spool, 0.3, 0.15, 15.0);
    println!("Steady yaw: open = {yaw_open:.3} rad/s, spool = {yaw_spool:.3} rad/s");
    assert!(yaw_spool < yaw_open * 0.97, "spool {yaw_spool:.3} must turn less than open {yaw_open:.3}");
}

/// Scenario: LSD power lock shapes corner-exit balance (Spec 042)
///
/// Given an RWD car with LSD power_lock 0.0 and then 0.8
/// When it drives through a corner on power, below the spin limit
/// Then the yaw rates differ by at least 10% (the locked diff feeds the slower inner wheel)
///
/// Measured: at full throttle the sports car spins with either setting (sideslip ~1.5 rad), so
/// the lock cannot matter there; on partial power it moves yaw by 13-17%.
#[test]
fn test_lsd_power_lock_changes_corner_exit_yaw() {
    let lsd = |power_lock: f32| DifferentialType::LimitedSlip { power_lock, coast_lock: 0.3, preload_nm: 0.0 };
    let yaw_free = steady_yaw_rate(lsd(0.0), 0.3, 0.15, 15.0);
    let yaw_locked = steady_yaw_rate(lsd(0.8), 0.3, 0.15, 15.0);
    let diff = (yaw_locked - yaw_free).abs() / yaw_free.max(1e-3);
    println!("Exit yaw: power_lock 0.0 = {yaw_free:.3}, 0.8 = {yaw_locked:.3} (diff {:.1}%)", diff * 100.0);
    assert!(diff >= 0.10, "power_lock must move exit yaw by >= 10% (got {:.1}%)", diff * 100.0);
}
