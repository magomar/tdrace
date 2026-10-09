//! Spec 104 (`specs/104_autocross_and_rallycross_tournament_sprint_weekend_format.md`): the stage
//! completion and finish-line timing a sprint weekend reads from the headless world.

use arcade_race_core::track::{create_prototypical_track, RaceDirection, Track, TrackProgressTracker, TrackShape};
use race_kit::ai::{BotAiDriver, BotProfile};
use race_kit::{DnfCause, DriveControls, FinishState, RaceFormat, RaceRules, RaceWorld, StageOutcome};
use wheelbase::{Car, CarConfig};

const DT: f32 = 1.0 / 120.0;
const GRID: usize = 8;

fn eight_car_heat(track: &Track, laps: u32) -> (RaceWorld<Car>, Vec<BotAiDriver>) {
    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules { format: RaceFormat::Laps(laps), ..RaceRules::default() });
    let mut drivers = Vec::new();
    for i in 0..GRID {
        let slot = track.grid_positions[i];
        world.spawn(
            Car::new(CarConfig::sports_car()).with_pose(slot.position, slot.angle),
            TrackProgressTracker::new(track.checkpoints.len(), 3),
        );
        drivers.push(BotAiDriver::with_seed(BotProfile::pro(), 100 + i as u64));
    }
    (world, drivers)
}

fn step_all(world: &mut RaceWorld<Car>, drivers: &mut [BotAiDriver], track: &Track) {
    let controls: Vec<DriveControls> = (0..GRID)
        .map(|i| {
            let others: Vec<&Car> = world.vehicles.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, c)| c).collect();
            drivers[i].compute_controls(&world.vehicles[i], track, &others, DT)
        })
        .collect();
    world.step(track, &controls, DT);
}

/// Scenario: A heat of 8 cars completes with real finish-line times
///
/// Given 8 bot-driven cars in a 2-lap sprint on the prototypical oval
/// When the world steps until the stage is complete
/// Then is_stage_complete is false while a car is still racing and true after the last one crosses,
/// every car is Finished with a real time, and positions follow the times
#[test]
fn a_heat_of_eight_completes_with_real_times() {
    let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    assert!(track.grid_positions.len() >= GRID, "the oval has an 8-car grid");
    let (mut world, mut drivers) = eight_car_heat(&track, 2);
    assert!(!world.is_stage_complete(), "nobody has crossed the line at the start");

    let mut incomplete_with_a_finisher = false;
    for _ in 0..(900.0 / DT) as usize {
        step_all(&mut world, &mut drivers, &track);
        let finished = (0..GRID).filter(|&i| world.is_finished(i)).count();
        if finished > 0 && finished < GRID {
            incomplete_with_a_finisher |= !world.is_stage_complete();
        }
        if world.is_stage_complete() {
            break;
        }
    }
    assert!(world.is_stage_complete(), "all 8 cars must finish: {:?}", world.finish);
    assert!(incomplete_with_a_finisher, "the stage is not complete while a car is still racing");

    let classification = world.stage_classification(&track);
    assert_eq!(classification.len(), GRID);
    for (k, row) in classification.iter().enumerate() {
        assert_eq!(row.position, k + 1);
        assert_eq!(row.outcome, StageOutcome::Finished);
        assert!(row.time.is_some(), "a finisher has a real time");
    }
    assert!(classification.windows(2).all(|w| w[0].time <= w[1].time), "times go up with position");
}

/// Scenario: A wrecked car classifies last and carries no time
///
/// Given a heat in which car 0 is wrecked at once and the other 7 finish
/// When the stage is classified
/// Then car 0 is in position 8 with outcome Dnf and no time, and the stage is complete
#[test]
fn a_wrecked_car_is_last_and_has_no_time() {
    let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    let (mut world, mut drivers) = eight_car_heat(&track, 2);
    world.finish[0] = FinishState::Dnf { time: 0.5, cause: DnfCause::Impact };
    for _ in 0..(900.0 / DT) as usize {
        step_all(&mut world, &mut drivers, &track);
        if (1..GRID).all(|i| world.is_finished(i)) {
            break;
        }
    }
    assert!((1..GRID).all(|i| world.is_finished(i)), "the other 7 cars finish: {:?}", world.finish);
    assert!(world.is_stage_complete(), "a wreck does not keep the stage open");

    let classification = world.stage_classification(&track);
    let last = classification.last().unwrap();
    assert_eq!((last.car, last.position, last.outcome, last.time), (0, GRID, StageOutcome::Dnf, None));
    assert!(classification[..GRID - 1].iter().all(|r| r.outcome == StageOutcome::Finished));
}

/// Scenario: A stage read mid-race is not complete and has no time for the cars on track
///
/// Given a heat 5 seconds after the start
/// When the stage is classified
/// Then is_stage_complete is false and every row is Unfinished with no time
#[test]
fn a_stage_read_mid_race_is_incomplete() {
    let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    let (mut world, mut drivers) = eight_car_heat(&track, 2);
    for _ in 0..(5.0 / DT) as usize {
        step_all(&mut world, &mut drivers, &track);
    }
    assert!(!world.is_stage_complete());
    let classification = world.stage_classification(&track);
    assert!(classification.iter().all(|r| r.outcome == StageOutcome::Unfinished && r.time.is_none()));
}

/// Scenario: The same heat replays bit for bit
///
/// Given two worlds built the same way
/// When both run 20 seconds
/// Then their classifications are identical
#[test]
fn a_heat_is_deterministic() {
    let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    let run = || {
        let (mut world, mut drivers) = eight_car_heat(&track, 2);
        for _ in 0..(20.0 / DT) as usize {
            step_all(&mut world, &mut drivers, &track);
        }
        world.stage_classification(&track)
    };
    assert_eq!(run(), run());
}
