//! Spec 083 (EvoRL bot performance calibration): the launch circuit table, the genome helper and the
//! `bot_eval_worker` evaluation bridge.
//! See `specs/083_evorl_bot_driving_style_and_performance_optimization.md`.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use tdrace_app::ai::bot_harness::{eval_driver_seed, launch_circuit_cars, run_harness_race, BotGenome, HarnessEntry};
use tdrace_app::ai::{BotAiDriver, BotProfile, DriverQuality, DriverTier, DrivingStyle, StyleTuning};
use tdrace_app::module::classic::ClassicGameModule;

const TIERS: [DriverTier; 5] = [DriverTier::Rookie, DriverTier::Amateur, DriverTier::Contender, DriverTier::Pro, DriverTier::Legend];

#[test]
fn test_launch_circuits_cover_the_four_launch_modules() {
    let circuits = launch_circuit_cars();
    for (module, expected) in [("classic", 18), ("kart", 20), ("autocross", 17), ("rally", 20)] {
        let n = circuits.iter().filter(|c| c.module == module).count();
        assert_eq!(n, expected, "{module} circuits");
        assert_eq!(n, tdrace_core::catalog::module_circuits(module).count(), "{module} matches the catalog");
    }
    assert_eq!(circuits.len(), 75);
}

#[test]
fn test_empty_genome_gives_the_game_profile() {
    for style in DrivingStyle::ALL {
        for tier in TIERS {
            let game = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(tier));
            assert_eq!(BotGenome::default().profile(style, tier).unwrap(), game, "{style:?} {tier:?}");
        }
    }
}

#[test]
fn test_stage_a_genome_moves_every_style_by_the_same_change() {
    let base = StyleTuning::for_style(DrivingStyle::Balanced);
    let genome = BotGenome { steering_kp: Some(base.steering_kp + 0.3), brake_margin: Some(base.brake_margin + 0.05), ..Default::default() };
    for style in DrivingStyle::ALL {
        let old = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Pro));
        let new = genome.profile(style, DriverTier::Pro).unwrap();
        assert!((new.steering_kp - old.steering_kp - 0.3).abs() < 1e-5, "{style:?} kp");
        assert!((new.brake_margin - old.brake_margin - 0.05).abs() < 1e-5, "{style:?} brake");
        assert_eq!(new.avoidance_distance, old.avoidance_distance, "{style:?} avoidance unchanged");
    }
}

#[test]
fn test_stage_b_genome_sets_the_tier_quality() {
    let genome = BotGenome { pace_limit: Some(0.93), consistency: Some(0.70), composure: Some(0.65), brake_padding: Some(0.10), ..Default::default() };
    let mut quality = DriverQuality::for_tier(DriverTier::Amateur);
    quality.pace_limit = 0.93;
    quality.consistency = 0.70;
    quality.composure = 0.65;
    quality.brake_padding = 0.10;
    let expected = BotProfile::from_style_and_quality(DrivingStyle::Smooth, &quality);
    assert_eq!(genome.profile(DrivingStyle::Smooth, DriverTier::Amateur).unwrap(), expected);
}

#[test]
fn test_genome_values_are_clamped_and_non_finite_values_fail() {
    let wild = BotGenome { steering_kp: Some(99.0), lookahead_time: Some(-5.0), ..Default::default() };
    let at_bounds = BotGenome { steering_kp: Some(3.50), lookahead_time: Some(0.20), ..Default::default() };
    assert_eq!(wild.profile(DrivingStyle::Balanced, DriverTier::Pro).unwrap(), at_bounds.profile(DrivingStyle::Balanced, DriverTier::Pro).unwrap());
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let genome = BotGenome { composure: Some(bad), ..Default::default() };
        assert!(genome.profile(DrivingStyle::Balanced, DriverTier::Pro).is_err());
    }
}

/// A running `bot_eval_worker` process.
struct Worker {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Worker {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_bot_eval_worker")).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("start worker");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Self { child, stdin, stdout }
    }

    fn ask(&mut self, line: &str) -> serde_json::Value {
        writeln!(self.stdin, "{line}").unwrap();
        self.stdin.flush().unwrap();
        let mut answer = String::new();
        self.stdout.read_line(&mut answer).expect("worker answer");
        serde_json::from_str(&answer).expect("worker answer is JSON")
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

const COASTAL_PAIR: &str = r#"{"circuit":"classic/gt_coastal_grand_prix","laps":3,"seed":17,"subject":{"style":"balanced","tier":4},"opponents":[{"style":"balanced","tier":4}]}"#;

/// Scenario: Worker returns the same result as the Rust harness
#[test]
fn test_worker_matches_the_rust_harness_and_is_deterministic() {
    let track = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
    let car = ClassicGameModule::car_classic_gt();
    let profile = BotProfile::from_style_and_quality(DrivingStyle::Balanced, &DriverQuality::for_tier(DriverTier::Pro));
    let entries = (0..2).map(|i| HarnessEntry::bot(BotAiDriver::with_seed(profile, eval_driver_seed(17, i)), car)).collect();
    let direct = run_harness_race(&track, entries, 3, 600.0);

    let mut worker = Worker::start();
    let first = worker.ask(COASTAL_PAIR);
    assert!(first["error"].is_null(), "{first}");
    assert_eq!(first["controls_hash"].as_u64().unwrap(), direct[0].controls_hash);
    let laps: Vec<f32> = first["lap_times"].as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect();
    assert_eq!(laps, direct[0].lap_times);
    assert_eq!(first["opponents"][0]["mean_flying_lap"].as_f64().unwrap() as f32, direct[1].mean_flying_lap().unwrap());

    let second = worker.ask(COASTAL_PAIR);
    assert_eq!(second["controls_hash"], first["controls_hash"]);
    assert_eq!(second["lap_times"], first["lap_times"]);
}

/// Scenario: Worker survives bad input
#[test]
fn test_worker_answers_bad_input_with_an_error_and_keeps_running() {
    let mut worker = Worker::start();
    let bad = [
        "this is not json",
        r#"{"circuit":"kart/no_such_track","laps":1,"seed":1,"subject":{"style":"balanced","tier":4}}"#,
        r#"{"circuit":"kart/lonato","car":"no_such_car","laps":1,"seed":1,"subject":{"style":"balanced","tier":4}}"#,
        r#"{"circuit":"kart/lonato","laps":1,"seed":1,"subject":{"style":"balanced","tier":4,"genome":{"steering_kp":NaN}}}"#,
        r#"{"circuit":"kart/lonato","laps":1,"seed":1,"subject":{"style":"balanced","tier":4,"genome":{"steering_kp":1e999}}}"#,
        r#"{"circuit":"kart/lonato","laps":1,"seed":1,"subject":{"style":"wild","tier":4}}"#,
        r#"{"circuit":"kart/lonato","laps":1,"seed":1,"subject":{"style":"balanced","tier":9}}"#,
        r#"{"circuit":"kart/lonato","laps":1,"seed":1,"subject":{"style":"balanced","tier":4},"subject_slot":3}"#,
    ];
    for line in bad {
        let answer = worker.ask(line);
        assert!(answer["error"].is_string(), "{line} -> {answer}");
    }
    let good = worker.ask(r#"{"circuit":"kart/lonato","laps":1,"seed":1,"subject":{"style":"balanced","tier":4}}"#);
    assert!(good["error"].is_null(), "{good}");
    assert_eq!(good["lap_times"].as_array().unwrap().len(), 1);
}
