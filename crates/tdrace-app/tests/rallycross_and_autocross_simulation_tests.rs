//! Comprehensive Multi-Circuit Physics Diagnostics & Lap Time Simulation
//! for Autocross (T1-T5) and Rallycross (T1-T7) vehicles.

use glam::Vec2;
use tdrace_app::ai::bot_harness::{run_harness_race, HarnessEntry};
use tdrace_app::ai::{BotAiDriver, BotProfile, DriverQuality, DriverTier, DrivingStyle};
use tdrace_app::catalog::find_model_by_id;
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::surface::SurfaceType;

const DT: f32 = 1.0 / 60.0;

fn measure_braking(cfg: &tdrace_core::physics::config::CarConfig, surface: SurfaceType) -> (f32, f32) {
    let mut car = Car::new(cfg.clone()).with_pose(Vec2::ZERO, 0.0);
    car.set_velocity(Vec2::new(100.0 / 3.6, 0.0));
    let mut dist = 0.0f32;
    let mut time = 0.0f32;
    let ctrl = CarControls::new(0.0, 0.0, 1.0, false);
    for _ in 0..600 {
        if car.state().speed < 0.5 {
            break;
        }
        let v_prev = car.state().speed;
        car.step(&ctrl, surface, DT);
        dist += (v_prev + car.state().speed) * 0.5 * DT;
        time += DT;
    }
    let avg_decel_g = (100.0 / 3.6) / (time.max(0.01) * 9.81);
    (avg_decel_g, dist)
}

fn measure_turning(cfg: &tdrace_core::physics::config::CarConfig, surface: SurfaceType, speed_kmh: f32) -> (f32, f32) {
    let mut car = Car::new(cfg.clone()).with_pose(Vec2::ZERO, 0.0);
    car.set_velocity(Vec2::new(speed_kmh / 3.6, 0.0));
    let ctrl = CarControls::new(0.50, 0.75, 0.0, false);
    for _ in 0..120 {
        car.step(&ctrl, surface, DT);
    }
    let speed = car.state().speed;
    let yaw = car.state().angular_velocity.abs();
    let radius = if yaw > 1e-3 { speed / yaw } else { 999.0 };
    let lat_g = (speed * yaw) / 9.81;
    (lat_g, radius)
}

fn measure_exit_slip(cfg: &tdrace_core::physics::config::CarConfig, surface: SurfaceType, speed_kmh: f32) -> (f32, f32, f32) {
    let mut car = Car::new(cfg.clone()).with_pose(Vec2::ZERO, 0.0);
    car.set_velocity(Vec2::new(speed_kmh / 3.6, 0.0));
    let ctrl = CarControls::new(1.0, 0.75, 0.0, false);
    let mut sum_yaw = 0.0f32;
    let mut sum_f = 0.0f32;
    let mut sum_r = 0.0f32;
    for _ in 0..120 {
        car.step(&ctrl, surface, DT);
        sum_yaw += car.state().angular_velocity.abs();
        let f = (car.state().wheels[0].slip_angle.abs() + car.state().wheels[1].slip_angle.abs()) * 0.5 * 57.2958;
        let r = (car.state().wheels[2].slip_angle.abs() + car.state().wheels[3].slip_angle.abs()) * 0.5 * 57.2958;
        sum_f += f;
        sum_r += r;
    }
    let avg_yaw = sum_yaw / 120.0;
    let exit_lat_g = (car.state().speed * avg_yaw) / 9.81;
    (exit_lat_g, sum_f / 120.0, sum_r / 120.0)
}

fn simulate_laps(module: &str, track_id: &str, car_cfg: tdrace_core::physics::config::CarConfig) -> (f32, f32, u32) {
    let track = tdrace_core::catalog::official_track(module, track_id);
    let quality = DriverQuality::for_tier(DriverTier::Pro);
    let profile = BotProfile::from_style_and_quality(DrivingStyle::Balanced, &quality);
    let bot = BotAiDriver::with_seed(profile, 42);
    let entry = HarnessEntry::bot(bot, car_cfg);
    let results = run_harness_race(&track, vec![entry], 2, 240.0);
    let res = &results[0];
    let l1 = res.lap_times.first().copied().unwrap_or(0.0);
    let flying = res.mean_flying_lap().unwrap_or(0.0);
    (l1, flying, res.wall_contacts)
}

#[test]
fn test_comprehensive_autocross_multi_circuit() {
    let cars = [
        (1, "autocross_ardennes_junior_t1"),
        (2, "autocross_ardennes_pro_t2"),
        (3, "autocross_petersen_buggy1600_t3"),
        (4, "autocross_bohemia_veloce_t4"),
        (5, "autocross_petersen_superbuggy_t5"),
    ];

    let circuits = [
        ("nova_paka_ax", "Nova Paka"),
        ("matschenberg_ax", "Matschenberg"),
        ("seelow_ax", "Seelow"),
        ("maggiora_ax", "Maggiora"),
        ("prerov_ax", "Prerov"),
    ];

    println!("\n=========================================================================================");
    println!("                           AUTOCROSS MULTI-CIRCUIT BENCHMARK                             ");
    println!("=========================================================================================");
    println!("{:<4} {:<28} {:<5} {:<6} {:<9} {:<7} {:<7} {:<7} {:<8} {:<8} {:<8}",
        "Tier", "Vehicle", "BHP", "Mass", "Drivetrain", "Grip", "Brake(N)", "Decel(G)", "Stop(m)", "TurnG", "TurnRad");

    for (tier, id) in cars {
        let model = find_model_by_id(id).unwrap();
        let cfg = model.to_car_config();
        let (decel, stop_dist) = measure_braking(&cfg, SurfaceType::Dirt);
        let (turn_g, turn_rad) = measure_turning(&cfg, SurfaceType::Dirt, 60.0);
        let (exit_g, f_slip, r_slip) = measure_exit_slip(&cfg, SurfaceType::Dirt, 60.0);

        println!("T{:<3} {:<28} {:<5} {:<6.0} {:<9} {:<7.2} {:<7.0} {:<7.2} {:<8.1} {:<8.2} {:<8.1}",
            tier, model.name, model.bhp, cfg.mass, format!("bias={:.2}", cfg.drive_bias),
            cfg.tire.grip, cfg.max_brake_force, decel, stop_dist, turn_g, turn_rad);
        println!("     -> Exit: LatG={:.2}g, FrontSlip={:.1}°, RearSlip={:.1}°", exit_g, f_slip, r_slip);

        print!("     -> Lap Times: ");
        for (track_id, track_name) in circuits {
            let (l1, flying, walls) = simulate_laps("autocross", track_id, cfg.clone());
            print!("| {}: L1={:.1}s Fly={:.2}s (W:{}) ", track_name, l1, flying, walls);
        }
        println!();
    }
}

#[test]
fn test_comprehensive_rallycross_multi_circuit() {
    let cars = [
        (1, "rally_gallia_200_t1"),
        (2, "rally_nordic_rx_lites_t2"),
        (3, "rally_hanguk_apex_t3"),
        (4, "rally_gallia_lyon_t4"),
        (5, "rally_gallia_volt_t5"),
        (6, "rally_nordic_valkyrie_t6"),
        (7, "rally_vortek_turbo_quattro_t7"),
    ];

    let circuits = [
        ("holjes_rx", "Höljes"),
        ("estering_rx", "Estering"),
        ("loheac_rx", "Lohéac"),
        ("catalunya_rx", "Catalunya"),
        ("lydden_hill", "Lydden Hill"),
        ("hell_rx", "Hell"),
    ];

    println!("\n=========================================================================================");
    println!("                           RALLYCROSS MULTI-CIRCUIT BENCHMARK                            ");
    println!("=========================================================================================");
    println!("{:<4} {:<32} {:<5} {:<6} {:<10} {:<7} {:<8} {:<8} {:<8} {:<8}",
        "Tier", "Vehicle", "BHP", "Mass", "Drive", "Grip", "Brake(N)", "Decel(G)", "Stop(m)", "TurnG");

    for (tier, id) in cars {
        let model = find_model_by_id(id).unwrap();
        let cfg = model.to_car_config();
        // Measure braking on Dirt and Asphalt
        let (decel_dirt, stop_dirt) = measure_braking(&cfg, SurfaceType::Dirt);
        let (decel_asph, stop_asph) = measure_braking(&cfg, SurfaceType::Asphalt);
        let (turn_g, turn_rad) = measure_turning(&cfg, SurfaceType::Dirt, 60.0);
        let (exit_g, f_slip, r_slip) = measure_exit_slip(&cfg, SurfaceType::Dirt, 60.0);

        println!("T{:<3} {:<32} {:<5} {:<6.0} {:<10} {:<7.2} {:<8.0} {:<8.2} {:<8.1} {:<8.2}",
            tier, model.name, model.bhp, cfg.mass, format!("bias={:.2}", cfg.drive_bias),
            cfg.tire.grip, cfg.max_brake_force, decel_dirt, stop_dirt, turn_g);
        println!("     -> Brake: Dirt={:.2}G ({:.1}m), Asphalt={:.2}G ({:.1}m) | TurnRad(60)={:.1}m",
            decel_dirt, stop_dirt, decel_asph, stop_asph, turn_rad);
        println!("     -> Exit: LatG={:.2}g, FrontSlip={:.1}°, RearSlip={:.1}°", exit_g, f_slip, r_slip);

        print!("     -> Lap Times: ");
        for (track_id, track_name) in circuits {
            let (l1, flying, walls) = simulate_laps("rally", track_id, cfg.clone());
            print!("| {}: L1={:.1}s Fly={:.2}s (W:{}) ", track_name, l1, flying, walls);
        }
        println!();
    }
}

#[test]
fn test_calibrated_rallycross_multi_circuit() {
    let cars = [
        (4, "rally_gallia_lyon_t4", "T4 Gallia Lyon WRX (600 BHP)"),
        (6, "rally_nordic_valkyrie_t6", "T6 Nordic Valkyrie (1070 BHP)"),
    ];

    let circuits = [
        ("holjes_rx", "Höljes"),
        ("estering_rx", "Estering"),
        ("loheac_rx", "Lohéac"),
        ("catalunya_rx", "Catalunya"),
        ("lydden_hill", "Lydden Hill"),
        ("hell_rx", "Hell"),
    ];

    println!("\n=========================================================================================");
    println!("                   CALIBRATED RALLYCROSS MULTI-CIRCUIT BENCHMARK                         ");
    println!("=========================================================================================");

    for (tier, id, label) in cars {
        let model = find_model_by_id(id).unwrap();
        let mut cfg = model.to_car_config();

        // 1. Calibrate competition brake system: 22,000 N base (scales with mass and brake rating)
        let mass_ratio = (model.weight_kg as f32) / 1250.0;
        let brake_mult = (0.80 + 0.40 * model.stats.4).clamp(0.5, 1.5);
        cfg.max_brake_force = 22500.0 * mass_ratio * brake_mult;

        // 2. Calibrate competition RX tire compound grip: 1.20 base -> 1.35 final
        cfg.tire.grip = (1.20 * (0.85 + 0.30 * model.stats.2)).clamp(0.5, 1.8);

        // 3. Calibrate high-power AWD torque split to 42:58 (0.42 front bias) and neutral rear grip
        cfg.drive_bias = 0.42;
        cfg.rear_axle.grip_scale = 1.00;

        // 4. Calibrate tractive force for extreme electric hypercar (T6)
        if tier == 6 {
            cfg.max_engine_force = (model.bhp as f32) * 14.5;
        }

        cfg = cfg.finalized();

        let (decel_dirt, stop_dirt) = measure_braking(&cfg, SurfaceType::Dirt);
        let (decel_asph, stop_asph) = measure_braking(&cfg, SurfaceType::Asphalt);
        let (turn_g, turn_rad) = measure_turning(&cfg, SurfaceType::Dirt, 60.0);

        println!("{:<30} Mass={:.0}kg Grip={:.2} Brake={:.0}N", label, cfg.mass, cfg.tire.grip, cfg.max_brake_force);
        println!("     -> Decel: Dirt={:.2}G ({:.1}m), Asphalt={:.2}G ({:.1}m) | TurnRad(60)={:.1}m, TurnG={:.2}g",
            decel_dirt, stop_dirt, decel_asph, stop_asph, turn_rad, turn_g);

        print!("     -> Lap Times: ");
        for (track_id, track_name) in circuits {
            let (l1, flying, walls) = simulate_laps("rally", track_id, cfg.clone());
            print!("| {}: L1={:.1}s Fly={:.2}s (W:{}) ", track_name, l1, flying, walls);
        }
        println!();
    }
}

#[test]
fn test_calibrated_autocross_multi_circuit() {
    let cars = [
        (3, "autocross_petersen_buggy1600_t3", "T3 Buggy 1600 (265 BHP)"),
        (4, "autocross_bohemia_veloce_t4", "T4 Bohemia Veloce TAX (550 BHP)"),
        (5, "autocross_petersen_superbuggy_t5", "T5 SuperBuggy V8 (680 BHP)"),
    ];

    let circuits = [
        ("nova_paka_ax", "Nova Paka"),
        ("matschenberg_ax", "Matschenberg"),
        ("seelow_ax", "Seelow"),
        ("maggiora_ax", "Maggiora"),
        ("prerov_ax", "Prerov"),
    ];

    println!("\n=========================================================================================");
    println!("                   CALIBRATED AUTOCROSS MULTI-CIRCUIT BENCHMARK                          ");
    println!("=========================================================================================");

    for (tier, id, label) in cars {
        let model = find_model_by_id(id).unwrap();
        let mut cfg = model.to_car_config();

        match tier {
            3 => {
                // T3: 42:58 split, neutral rear grip
                cfg.drive_bias = 0.42;
                cfg.rear_axle.grip_scale = 1.00;
            }
            4 => {
                // T4: 22,500 N competition brakes, 1.20 base tire grip, 45:55 Sadev AWD split, neutral rear grip
                let mass_ratio = (model.weight_kg as f32) / 1250.0;
                let brake_mult = (0.80 + 0.40 * model.stats.4).clamp(0.5, 1.5);
                cfg.max_brake_force = 22500.0 * mass_ratio * brake_mult;
                cfg.tire.grip = (1.20 * (0.85 + 0.30 * model.stats.2)).clamp(0.5, 1.8);
                cfg.drive_bias = 0.45;
                cfg.rear_axle.grip_scale = 1.00;
            }
            5 => {
                // T5: 18,000 N brakes, 42:58 split, neutral rear grip, wide 280mm rears
                let mass_ratio = (model.weight_kg as f32) / 700.0;
                let brake_mult = (0.80 + 0.40 * model.stats.4).clamp(0.5, 1.5);
                cfg.max_brake_force = 18000.0 * mass_ratio * brake_mult;
                cfg.drive_bias = 0.42;
                cfg.rear_axle.grip_scale = 1.00;
                cfg.wheels[2].tire_width = 0.28;
                cfg.wheels[3].tire_width = 0.28;
                cfg.wheels[2].rotational_inertia = 1.45;
                cfg.wheels[3].rotational_inertia = 1.45;
                cfg.max_engine_force = (model.bhp as f32) * 14.5;
            }
            _ => {}
        }

        cfg = cfg.finalized();

        let (decel_dirt, stop_dirt) = measure_braking(&cfg, SurfaceType::Dirt);
        let (turn_g, turn_rad) = measure_turning(&cfg, SurfaceType::Dirt, 60.0);
        let (exit_g, f_slip, r_slip) = measure_exit_slip(&cfg, SurfaceType::Dirt, 60.0);

        println!("{:<35} Mass={:.0}kg Grip={:.2} Brake={:.0}N Decel={:.2}G ({:.1}m)",
            label, cfg.mass, cfg.tire.grip, cfg.max_brake_force, decel_dirt, stop_dirt);
        println!("     -> Exit: LatG={:.2}g, FrontSlip={:.1}°, RearSlip={:.1}° | TurnRad(60)={:.1}m, TurnG={:.2}g",
            exit_g, f_slip, r_slip, turn_rad, turn_g);

        print!("     -> Lap Times: ");
        for (track_id, track_name) in circuits {
            let (l1, flying, walls) = simulate_laps("autocross", track_id, cfg.clone());
            print!("| {}: L1={:.1}s Fly={:.2}s (W:{}) ", track_name, l1, flying, walls);
        }
        println!();
    }
}

