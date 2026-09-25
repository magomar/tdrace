use std::env;
use std::fs;
use std::process;
use wheelbase::config::CarConfig;
use wheelbase::sim::optimizer::{
    standard_tuning_bounds, AutoCalibrator, AutoTunerConfig, CalibrationTarget,
    DrivetrainConstraint,
};

fn print_usage() {
    println!(
        r#"Automated Vehicle Dynamic Calibration Optimizer (Spec 035)

USAGE:
    cargo run --bin auto_tune -- [OPTIONS]

OPTIONS:
    --vehicle <NAME>        Vehicle archetype: kart, sports_car, drift_car, rally_car, stock_car (default: kart)
    --constraint <TYPE>     Drivetrain constraint: spool, salisbury, multi_lsd, open (default: spool)
    --target <NAME>         Calibration target preset: kart_sprint, gt3, nascar_ta1, rx_supercar (default: kart_sprint)
    --generations <N>       Maximum CMA-ES generations (default: 30)
    --population <N>        Population size per generation (default: 16)
    --seed <N>              Deterministic RNG seed (default: 42)
    --report <PATH>         Write markdown receipt to specified file path
    --help                  Print this help message
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut vehicle_arg = "kart".to_string();
    let mut constraint_arg = "spool".to_string();
    let mut target_arg = "kart_sprint".to_string();
    let mut generations: usize = 30;
    let mut population: usize = 16;
    let mut seed: u64 = 42;
    let mut report_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--vehicle" => {
                i += 1;
                if i < args.len() {
                    vehicle_arg = args[i].clone();
                }
            }
            "--constraint" => {
                i += 1;
                if i < args.len() {
                    constraint_arg = args[i].clone();
                }
            }
            "--target" => {
                i += 1;
                if i < args.len() {
                    target_arg = args[i].clone();
                }
            }
            "--generations" => {
                i += 1;
                if i < args.len() {
                    generations = args[i].parse().unwrap_or(30);
                }
            }
            "--population" => {
                i += 1;
                if i < args.len() {
                    population = args[i].parse().unwrap_or(16);
                }
            }
            "--seed" => {
                i += 1;
                if i < args.len() {
                    seed = args[i].parse().unwrap_or(42);
                }
            }
            "--report" => {
                i += 1;
                if i < args.len() {
                    report_path = Some(args[i].clone());
                }
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            unknown => {
                eprintln!("Unknown argument: {}", unknown);
                print_usage();
                process::exit(1);
            }
        }
        i += 1;
    }

    let (vehicle_name, base_config) = match vehicle_arg.as_str() {
        "kart" => ("Sprint Kart", CarConfig::kart()),
        "sports_car" => ("Sports Car", CarConfig::sports_car()),
        "drift_car" => ("Drift Machine", CarConfig::drift_car()),
        "rally_car" => ("Rally Supercar", CarConfig::rally_car()),
        "stock_car" => ("Cup Stock Car", CarConfig::stock_car_ta1()),
        "sand_rail" => ("Sand Rail Buggy", CarConfig::sand_rail()),
        other => {
            eprintln!("Unsupported vehicle '{}'. Using 'kart'.", other);
            ("Sprint Kart", CarConfig::kart())
        }
    };

    let constraint = match constraint_arg.as_str() {
        "spool" => match vehicle_arg.as_str() {
            "stock_car" => DrivetrainConstraint::SpoolAxle {
                min_caster_jacking_unloading_ratio: 0.0,
                max_turning_diameter_m: 11.2,
            },
            "sand_rail" => DrivetrainConstraint::SpoolAxle {
                min_caster_jacking_unloading_ratio: 0.0,
                max_turning_diameter_m: 7.5,
            },
            _ => DrivetrainConstraint::SpoolAxle {
                min_caster_jacking_unloading_ratio: 0.80,
                max_turning_diameter_m: 2.6,
            },
        },
        "salisbury" => DrivetrainConstraint::SalisburyRwd {
            min_power_coast_delta: 0.15,
            max_preload_nm: 120.0,
            max_yaw_acceleration_rad_s2: 3.5,
        },
        "multi_lsd" => DrivetrainConstraint::MultiLsdAwd {
            drive_bias_range: (0.35, 0.65),
            strict_torque_conservation: true,
        },
        "open" => DrivetrainConstraint::OpenDiff,
        other => {
            eprintln!("Unsupported constraint '{}'. Using 'spool'.", other);
            DrivetrainConstraint::SpoolAxle {
                min_caster_jacking_unloading_ratio: 0.80,
                max_turning_diameter_m: 2.6,
            }
        }
    };

    let target = match target_arg.as_str() {
        "kart_sprint" | "sprint_kart" => CalibrationTarget::sprint_kart(),
        "gt3" | "gt3_homologation" => CalibrationTarget::gt3_homologation(),
        "nascar_ta1" | "stock_car_ta1" => CalibrationTarget::stock_car_ta1(),
        "rx_supercar" | "rallycross_supercar" => CalibrationTarget::rallycross_supercar(),
        "sand_rail" | "sand_rail_buggy" => CalibrationTarget::sand_rail_buggy(),
        other => {
            eprintln!("Unsupported target '{}'. Using 'sprint_kart'.", other);
            CalibrationTarget::sprint_kart()
        }
    };

    println!(
        "Initiating automated calibration for '{}' [Constraint: {:?}, Target: {}]...",
        vehicle_name, constraint, target.name
    );
    println!(
        "Generations: {}, Population: {}, Seed: {}",
        generations, population, seed
    );

    let tuner_config = AutoTunerConfig {
        max_generations: generations,
        population_size: Some(population),
        convergence_epsilon: 1e-4,
        penalty_stiffness: 1.0,
        dt: 1.0 / 60.0,
        seed,
    };

    let bounds = standard_tuning_bounds(&constraint, &base_config);
    let calibrator = AutoCalibrator::new(tuner_config, constraint, target, bounds);

    let result = calibrator.calibrate(vehicle_name, &base_config);

    let report_markdown = result.to_markdown();
    println!("\n{}", report_markdown);

    if let Some(path) = report_path {
        if let Err(e) = fs::write(&path, &report_markdown) {
            eprintln!("Failed to write report to {}: {}", path, e);
            process::exit(1);
        } else {
            println!("Calibration report written to {}", path);
        }
    }
}
