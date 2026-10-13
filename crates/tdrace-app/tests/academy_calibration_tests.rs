//! Spec 064 and Spec 109 calibration: a headless reference run of every Academy mission, Tiers 1 to 3.
//!
//! The reference driver is a Contender balanced bot with no human mistakes. It drives the mission's
//! circuit and car with the mission's own rules (damage, wear scale, start wear, laps, joker rule,
//! opponents), and the run is scored by the same `evaluate_academy_attempt_on` the game uses.
//! `print_reference_table` (run with `--ignored --nocapture`, in release) prints the numbers the
//! mission targets are set from: Gold about 3% over the reference lap, Silver about 12%, Bronze about 25%.
//! A mission whose Bronze target the reference run cannot meet cleanly does not ship.

use race_kit::{RaceEvent, RaceFormat, RaceRules, RaceWorld};
use tdrace_app::ai::humanize::HumanTraits;
use tdrace_app::ai::{BotAiDriver, BotProfile, BotRaceState, BotRouteStrategy, DriverQuality, DriverTier, DrivingStyle};
use tdrace_app::catalog::CLASSIC_ARCADE_CARS;
use tdrace_app::game::academy::{
    evaluate_academy_attempt_on, AcademyChallengeState, AcademyLessonDef, AcademyLessonId, AcademyMedal,
    AcademyObjective,
};
use tdrace_app::game::joker_rule_for;
use tdrace_app::profile::PlayerProfile;
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::track::checkpoint::TrackProgressTracker;

const DT: f32 = 1.0 / 120.0;

/// What the reference run of one mission measured.
#[derive(Debug, Clone)]
struct Reference {
    finished: bool,
    /// Race time of the player in seconds, including a joker penalty.
    time: f32,
    /// Best lap of the player in seconds: the time Tier 1 lessons 1 to 3 are scored on.
    best_lap: f32,
    worst_wear: f32,
    /// Hardest landing as impulse (touchdown speed x 100).
    max_landing: f32,
    /// Best single drift banked.
    best_drift: f32,
    /// Largest drift score ever reached, banked or not.
    peak_drift: f32,
    /// Hardest wall or car impact as impulse (impact speed x 100).
    max_impact: f32,
    /// Longest time off the track, seconds.
    off_track_s: f32,
    position: u32,
    jokers: u32,
    joker_mandatory: u32,
}

/// Runs the reference driver through the mission as `RaceSession` sets it up.
fn run_reference(def: &AcademyLessonDef) -> Reference {
    run_reference_with(def, DrivingStyle::Balanced, DriverTier::Contender)
}

/// A scripted drift of the reference driver: from track distance `start_m` the throttle is full, the
/// handbrake is held for `handbrake_s`, and the steering of the line follower is scaled by `steer_gain`
/// for `hold_s` seconds. The line follower drives the rest of the lap.
#[derive(Debug, Clone, Copy)]
struct DriftScript {
    start_m: f32,
    handbrake_s: f32,
    hold_s: f32,
    steer_gain: f32,
    /// Hold the slide with `steer = base + countersteer * sideslip` instead of the line follower's steering.
    pd: Option<(f32, f32)>,
}

fn run_reference_with(def: &AcademyLessonDef, style: DrivingStyle, tier: DriverTier) -> Reference {
    run_reference_scripted(def, style, tier, None)
}

fn run_reference_scripted(def: &AcademyLessonDef, style: DrivingStyle, tier: DriverTier, script: Option<DriftScript>) -> Reference {
    let track = tdrace_core::catalog::official_track("classic", &def.track_slug);
    let model = CLASSIC_ARCADE_CARS.iter().find(|c| c.id == def.car_slug).expect("mission car exists");
    let rules = def.rules;
    let joker = joker_rule_for(&track);
    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules {
        format: RaceFormat::Laps(rules.laps),
        damage_enabled: rules.damage_enabled,
        joker,
        ..RaceRules::default()
    });
    let n = 1 + rules.bots;
    assert!(n <= track.grid_positions.len(), "{}: grid of {} too small", def.track_slug, track.grid_positions.len());

    // The player starts at the back of the grid: the worst slot to win from.
    let player_slot = n - 1;
    let mut drivers: Vec<BotAiDriver> = Vec::new();
    let mut order: Vec<usize> = (0..rules.bots).collect();
    order.push(usize::MAX); // the player
    let mut player_idx = 0;
    for (slot, who) in order.iter().enumerate() {
        let mut config = model.to_car_config();
        config.damage_enabled = rules.damage_enabled;
        config.wear_rate_scale = rules.wear_rate_scale;
        let spawn = track.grid_positions[slot];
        let mut car = Car::new(config).with_pose(spawn.position, spawn.angle);
        let is_player = *who == usize::MAX;
        if is_player {
            player_idx = slot;
            for wheel in car.state.wheel_assemblies.iter_mut() {
                wheel.wear = rules.start_wear;
            }
        }
        world.spawn(car, TrackProgressTracker::new(track.checkpoints.len(), 3));
        let driver = if is_player {
            let mut profile =
                BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(tier));
            profile.traits = HumanTraits::none();
            BotAiDriver::with_seed(profile, 64)
        } else {
            let style = DrivingStyle::ALL[slot % DrivingStyle::ALL.len()];
            BotAiDriver::with_seed(
                BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Rookie)),
                640 + slot as u64,
            )
        };
        let driver = if joker.mandatory > 0 {
            driver.with_route_strategy(BotRouteStrategy::RallycrossJoker { planned_joker_lap: 2, adaptive_traffic_undercut: false })
        } else {
            driver
        };
        drivers.push(driver);
    }
    assert_eq!(player_idx, player_slot);

    let mut r = Reference {
        finished: false,
        time: 0.0,
        best_lap: 0.0,
        worst_wear: 0.0,
        max_landing: 0.0,
        best_drift: 0.0,
        peak_drift: 0.0,
        max_impact: 0.0,
        off_track_s: 0.0,
        position: 0,
        jokers: 0,
        joker_mandatory: joker.mandatory,
    };
    let mut was_drifting = false;
    let mut drift_peak = 0.0f32;
    let mut script_started: Option<f32> = None;
    let mut sim_time = 0.0f32;
    let max_steps = (def.rules.laps as f32 * 300.0 / DT) as usize;
    for _ in 0..max_steps {
        let mut controls: Vec<CarControls> = Vec::with_capacity(n);
        for (i, driver) in drivers.iter_mut().enumerate() {
            let others: Vec<&Car> = world.vehicles.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, c)| c).collect();
            driver.sync_race_state(BotRaceState {
                lap: world.trackers[i].current_lap,
                jokers: world.jokers_taken(i),
                total_laps: rules.laps,
                mandatory_jokers: joker.mandatory,
            });
            let mut c = driver.compute_controls(&world.vehicles[i], &track, &others, DT);
            if let (true, Some(sc)) = (i == player_idx, script) {
                let progress = world.trackers[i].progress_distance;
                if script_started.is_none() && world.trackers[i].current_lap == 1 && progress >= sc.start_m && progress < sc.start_m + 40.0 {
                    script_started = Some(sim_time);
                }
                if let Some(t0) = script_started {
                    let t = sim_time - t0;
                    if t < sc.hold_s {
                        c.throttle = 1.0;
                        c.brake = 0.0;
                        // Kick the tail out first, then re-kick whenever the slide dies away.
                        let car = &world.vehicles[i];
                        c.handbrake = t < sc.handbrake_s || (car.state.sideslip_angle.abs() < 0.22 && car.state.speed > 8.0);
                        c.steer = match sc.pd {
                            Some((base, k)) => (base + k * car.state.sideslip_angle).clamp(-1.0, 1.0),
                            None => (c.steer * sc.steer_gain).clamp(-1.0, 1.0),
                        };
                    }
                }
            }
            controls.push(c);
        }
        sim_time += DT;
        let events: Vec<RaceEvent> = world.step(&track, &controls, DT).to_vec();
        for ev in events {
            match ev {
                RaceEvent::Landed { car, .. } if car == player_idx => {
                    r.max_landing = r.max_landing.max(world.vehicles[car].state.last_touchdown_speed * 100.0);
                }
                RaceEvent::WallImpact { car, event } if car == player_idx => {
                    r.max_impact = r.max_impact.max(event.impact_speed * 100.0);
                }
                RaceEvent::VehicleImpact(cev) if cev.car_a_idx == player_idx || cev.car_b_idx == player_idx => {
                    r.max_impact = r.max_impact.max(cev.closing_speed * 100.0);
                }
                _ => {}
            }
        }
        // A drift is banked when it ends, as RaceSession does.
        let car = &world.vehicles[player_idx];
        let drifting = car.state.is_drifting;
        drift_peak = drift_peak.max(car.state.drift_score);
        r.peak_drift = r.peak_drift.max(car.state.drift_score);
        if was_drifting && !drifting {
            // The Academy banks a drift when it ends, whatever its size (Spec 064 calibration).
            r.best_drift = r.best_drift.max(drift_peak);
            drift_peak = 0.0;
        }
        was_drifting = drifting;
        r.off_track_s = r.off_track_s.max(world.trackers[player_idx].off_track_timer);

        if world.trackers[player_idx].current_lap > rules.laps {
            let results = world.results(&track);
            let row = results.iter().find(|row| row.car == player_idx).expect("player row");
            r.finished = true;
            r.time = row.time + row.penalty;
            r.best_lap = world.trackers[player_idx].best_lap_time.unwrap_or(0.0);
            r.position = row.position as u32;
            r.jokers = row.jokers;
            break;
        }
    }
    r.worst_wear = world.vehicles[player_idx].state.wheel_assemblies.iter().map(|w| w.wear).fold(0.0, f32::max);
    r
}

/// Scores the reference run through the game's own evaluation, on a fresh graduated profile.
fn score(def: &AcademyLessonDef, r: &Reference) -> (AcademyMedal, bool) {
    let mut profile = PlayerProfile::new_rookie("Reference");
    profile.academy_progress.license_granted = true;
    let mut state = AcademyChallengeState::new(def.id);
    state.observe(r.worst_wear, r.best_drift);
    state.finish_position = r.position;
    // The player's own impacts count towards the clean-attempt rule as RaceSession counts them.
    if r.max_impact > 0.0 {
        state.register_collision(r.max_impact);
    }
    if r.max_landing > 0.0 {
        state.register_landing(r.max_landing);
    }
    state.set_off_track_seconds(r.off_track_s);
    let eval = evaluate_academy_attempt_on(&mut profile, def.id, r.time, &state, "2026-10-11");
    (eval.medal, eval.clean_attempt)
}

/// The scripted drift the reference driver takes in the drift mission: found by `search_drift_script`
/// (the best clean banked drift of the lap on `ax_clay_bowl`).
const DRIFT_REFERENCE_SCRIPT: DriftScript =
    DriftScript { start_m: 480.0, handbrake_s: 0.5, hold_s: 14.0, steer_gain: 1.0, pd: Some((0.3, 0.0)) };

/// The reference run of a mission: the line follower, with the scripted drift in the drift mission.
fn run_mission_reference(def: &AcademyLessonDef) -> Reference {
    match def.objective {
        AcademyObjective::DriftScore => {
            run_reference_scripted(def, DrivingStyle::Balanced, DriverTier::Contender, Some(DRIFT_REFERENCE_SCRIPT))
        }
        _ => run_reference(def),
    }
}

/// The reference run of a mission. Every mission runs once per test binary, in parallel threads.
fn reference_for(def: &AcademyLessonDef) -> Reference {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    static RUNS: OnceLock<HashMap<&'static str, Reference>> = OnceLock::new();
    let runs = RUNS.get_or_init(|| {
        let defs = missions();
        std::thread::scope(|scope| {
            let handles: Vec<_> = defs.iter().map(|d| (d.id.slug(), scope.spawn(move || run_mission_reference(d)))).collect();
            handles.into_iter().map(|(slug, h)| (slug, h.join().expect("reference run panicked"))).collect()
        })
    });
    runs[def.id.slug()].clone()
}

fn missions() -> Vec<AcademyLessonDef> {
    [1u8, 2, 3].iter().flat_map(|t| AcademyLessonDef::tier_curriculum(*t)).collect()
}

/// Prints what the targets are set from. Run: cargo test --release -p tdrace-app --test
/// academy_calibration_tests print_reference_table -- --ignored --nocapture
#[test]
#[ignore]
fn print_reference_table() {
    let lessons = missions();
    for def in lessons {
        let r = run_mission_reference(&def);
        let len = tdrace_core::catalog::official_track("classic", &def.track_slug).spline.total_length();
        println!(
            "{:<10} len={:6.0} fin={} time={:7.2} lap={:7.2} wear={:.3} landing={:6.1} drift={:7.1} impact={:6.1} offtrk={:4.1} pos={} jokers={}/{}",
            def.id.slug(),
            len,
            r.finished,
            r.time,
            r.best_lap,
            r.worst_wear,
            r.max_landing,
            r.best_drift,
            r.max_impact,
            r.off_track_s,
            r.position,
            r.jokers,
            r.joker_mandatory
        );
    }
}

/// Scenario: every mission can be passed cleanly
///
/// Given each Academy mission of Tiers 1 to 3 with its circuit, car and rules
/// When the reference driver runs it headless
/// Then the reference finishes, the attempt is clean, and it earns at least Bronze
/// (a mission whose Bronze target the reference run cannot meet cleanly does not ship)
#[test]
fn every_mission_bronze_is_reachable_cleanly_by_the_reference_run() {
    let mut failures = Vec::new();
    for def in missions() {
        let r = reference_for(&def);
        let (medal, clean) = score(&def, &r);
        println!("{}: {:?} clean={} {:?}", def.id.slug(), medal, clean, r);
        if !r.finished {
            failures.push(format!("{}: the reference run did not finish", def.id.slug()));
        } else if !clean || medal < AcademyMedal::Bronze {
            failures.push(format!("{}: reference earned {:?} (clean {}) {:?}", def.id.slug(), medal, clean, r));
        }
        if def.rules.joker_required {
            assert_eq!(r.joker_mandatory, 1, "{}: the joker rule is off on its circuit", def.id.slug());
            assert_eq!(r.jokers, 1, "{}: the reference run did not take the joker", def.id.slug());
        }
    }
    assert!(failures.is_empty(), "{:#?}", failures);
}

/// The medal targets are set from the reference lap: Gold about 3% over it, Silver about 12%, Bronze about 25%.
/// A drift mission mirrors that with Gold at the reference, Silver at 75% and Bronze at 50% of Gold (the ratio of
/// the Spec 064 initial values).
#[test]
fn medal_targets_follow_the_reference_run() {
    for def in missions() {
        let r = reference_for(&def);
        let near = |target: f32, expected: f32| (target / expected - 1.0).abs() < 0.015;
        match def.objective {
            AcademyObjective::BestTime | AcademyObjective::CleanLanding | AcademyObjective::WearCap { .. } => {
                assert!(near(def.gold_time_sec, r.time * 1.03), "{} Gold {} vs reference {}", def.id.slug(), def.gold_time_sec, r.time);
                assert!(near(def.silver_time_sec, r.time * 1.12), "{} Silver {} vs reference {}", def.id.slug(), def.silver_time_sec, r.time);
                assert!(near(def.bronze_time_sec, r.time * 1.25), "{} Bronze {} vs reference {}", def.id.slug(), def.bronze_time_sec, r.time);
            }
            AcademyObjective::DriftScore => {
                assert!(def.gold_time_sec <= r.best_drift && def.gold_time_sec > 0.9 * r.best_drift, "{} Gold {} vs reference {}", def.id.slug(), def.gold_time_sec, r.best_drift);
                assert!(near(def.silver_time_sec, def.gold_time_sec * 0.75), "{}", def.id.slug());
                assert!(near(def.bronze_time_sec, def.gold_time_sec * 0.5), "{}", def.id.slug());
            }
            AcademyObjective::Podium { .. } => {}
        }
        // The reference lap is a Gold run.
        let (medal, _) = score(&def, &r);
        assert_eq!(medal, AcademyMedal::Gold, "{}: {:?}", def.id.slug(), r);
    }
}

/// The landing limit of the clean-landing mission clears the reference run's hardest landing by about 25%.
#[test]
fn landing_limit_clears_the_reference_landings_with_a_margin() {
    let def = AcademyLessonDef::find(AcademyLessonId::Acad2RampBalance).unwrap();
    let r = reference_for(&def);
    assert!(r.max_landing > 0.0, "the circuit must have a ramp");
    let limit = AcademyChallengeState::MAX_PERMISSIBLE_LANDING_IMPULSE;
    assert!(r.max_landing < limit, "reference lands at {:.0}, limit {:.0}", r.max_landing, limit);
    assert!(limit < 1.4 * r.max_landing, "the limit must still ask for a clean landing: reference {:.0}, limit {:.0}", r.max_landing, limit);
}

/// Tyre discipline: a clean run barely wears the tyres under the Tier 3 scale, a run that slides a lot ends over the
/// wear cap. About 8 s of full slide at the Tier 3 scale does it.
#[test]
fn wear_cap_is_passed_by_clean_driving_and_broken_by_sliding() {
    use tdrace_core::physics::surface::SurfaceType;
    for def in missions().into_iter().filter(|d| matches!(d.objective, AcademyObjective::WearCap { .. } | AcademyObjective::Podium { .. })) {
        let max_wear = match def.objective {
            AcademyObjective::WearCap { max_wear } | AcademyObjective::Podium { max_wear, .. } => max_wear,
            _ => unreachable!(),
        };
        let r = reference_for(&def);
        assert!(r.worst_wear < 0.5 * max_wear, "{}: the reference wears {:.3} of {:.2}", def.id.slug(), r.worst_wear, max_wear);

        let model = CLASSIC_ARCADE_CARS.iter().find(|c| c.id == def.car_slug).unwrap();
        let mut config = model.to_car_config();
        config.wear_rate_scale = def.rules.wear_rate_scale;
        let mut car = Car::new(config);
        car.state.velocity = glam::Vec2::new(30.0, 0.0);
        for step in 0..(8.0 / DT) as usize {
            let c = CarControls { throttle: 1.0, steer: 0.5, handbrake: step as f32 * DT < 0.5, ..Default::default() };
            car.step(&c, SurfaceType::Asphalt, DT);
        }
        let slid = car.state.wheel_assemblies.iter().map(|w| w.wear).fold(0.0, f32::max);
        assert!(slid > max_wear, "{}: 8 s of slide wear {:.3}, cap {:.2}", def.id.slug(), slid, max_wear);
    }
}

/// The Tyre Conservation Sprint: the reference wins the race from the back of the grid, and a finish in last place
/// with the tyres under the cap still earns Bronze.
#[test]
fn tyre_conservation_sprint_is_won_by_the_reference_from_the_back_of_the_grid() {
    let def = AcademyLessonDef::find(AcademyLessonId::Acad3TyreConservation).unwrap();
    let r = reference_for(&def);
    assert_eq!(r.position, 1, "{:?}", r);
    let mut last = r.clone();
    last.position = 4;
    assert_eq!(score(&def, &last).0, AcademyMedal::Bronze);
}

/// Finds `DRIFT_REFERENCE_SCRIPT`: the scripted drift with the best clean banked score on the drift circuit.
/// Run: cargo test --release -p tdrace-app --test academy_calibration_tests search_drift_script -- --ignored --nocapture
#[test]
#[ignore]
fn search_drift_script() {
    let def = AcademyLessonDef::find(AcademyLessonId::Acad2DirtDrift).unwrap();
    let mut best: Option<(f32, DriftScript, Reference)> = None;
    let mut modes: Vec<(Option<(f32, f32)>, f32)> = vec![(None, 1.0), (None, 1.6)];
    for dir in [-1.0f32, 1.0] {
        for (base, k) in [(0.5f32, 1.5f32), (0.9, 3.0), (0.3, 0.0)] {
            modes.push((Some((dir * base, k)), 1.0));
        }
    }
    for start in (0..1000).step_by(40) {
        for hb in [0.5f32] {
            for hold in [8.0f32, 14.0] {
                for (pd, gain) in &modes {
                    let sc = DriftScript { start_m: start as f32, handbrake_s: hb, hold_s: hold, steer_gain: *gain, pd: *pd };
                    let r = run_reference_scripted(&def, DrivingStyle::Balanced, DriverTier::Contender, Some(sc));
                    let clean = r.max_impact <= 800.0 && r.off_track_s <= 2.0 && r.finished;
                    if clean && best.as_ref().is_none_or(|b| r.best_drift > b.0) {
                        best = Some((r.best_drift, sc, r));
                    }
                }
            }
        }
    }
    println!("best clean drift: {:?}", best);
}






/// Scenario: a player can graduate (Spec 109)
///
/// Given a rookie profile and the four Tier 1 lessons run in order at the reference pace
/// When Lesson 4 ends at Bronze or better
/// Then the profile holds Class D and Tier 2 is unlocked
#[test]
fn a_player_at_the_reference_pace_graduates_and_opens_tier_2() {
    let mut profile = PlayerProfile::new_rookie("Graduate");
    assert!(!profile.academy_progress.is_tier_unlocked(2));
    for def in AcademyLessonDef::tier_curriculum(1) {
        assert!(profile.academy_progress.is_lesson_unlocked(def.id), "{} is locked", def.id.slug());
        let r = reference_for(&def);
        let mut state = AcademyChallengeState::new(def.id);
        state.observe(r.worst_wear, r.best_drift);
        if r.max_impact > 0.0 {
            state.register_collision(r.max_impact);
        }
        let eval = evaluate_academy_attempt_on(&mut profile, def.id, r.time, &state, "2026-10-11");
        assert_eq!(eval.medal, AcademyMedal::Gold, "{}: {:?}", def.id.slug(), r);
    }
    assert!(profile.academy_progress.is_graduated());
    assert_eq!(profile.license_grade, tdrace_app::profile::LicenseGrade::ClassD);
    assert!(profile.academy_progress.is_tier_unlocked(2));
}
