//! Spec 055 (Classic Circuits Revamp): bots race every new Classic circuit.
//! See `specs/055_classic_circuits_revamp.md`, scenario "Every new circuit is valid and raceable".

use tdrace_app::ai::bot_harness::{run_harness_race, sample_bot, HarnessEntry, CLASSIC_CIRCUIT_CARS as CIRCUITS};
use tdrace_app::ai::{DriverTier, DrivingStyle};
use tdrace_app::module::classic::ClassicGameModule;




const STYLES: [DrivingStyle; 4] = [
    DrivingStyle::Balanced,
    DrivingStyle::Aggressive,
    DrivingStyle::Smooth,
    DrivingStyle::Calculating,
];

/// Longest time a bot may go without progress (tdrace-le75).
const MAX_NO_PROGRESS_S: f32 = 10.0;

/// Scenario: Every new circuit is valid and raceable
///
/// Given each new Classic circuit and its Classic car
/// When a full grid of 8 bots (4 Rookie, 4 Pro) races 3 laps in the bot harness
/// Then every bot finishes, and no bot goes MAX_NO_PROGRESS_S without progress (it would be stuck on a wall)
#[test]
fn test_bots_finish_three_laps_on_every_new_circuit() {
    let mut failures = Vec::new();
    for (id, car) in CIRCUITS {
        let track = tdrace_core::catalog::official_track("classic", id);
        let entries = (0..8)
            .map(|i| {
                let tier = if i % 2 == 0 { DriverTier::Rookie } else { DriverTier::Pro };
                HarnessEntry::bot(sample_bot(STYLES[i % STYLES.len()], tier, 55 + i as u64), car())
            })
            .collect();
        for (i, r) in run_harness_race(&track, entries, 3, 400.0).iter().enumerate() {
            if !r.finished || r.longest_no_progress_s >= MAX_NO_PROGRESS_S {
                failures.push(format!(
                    "{}: bot {} finished {} laps (longest stop {:.1} s, longest no-progress {:.1} s)",
                    id,
                    i,
                    r.lap_times.len(),
                    r.longest_slow_s,
                    r.longest_no_progress_s
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{:#?}", failures);
}


/// Scenario: a car is never trapped on its own category's circuit (tdrace-le75)
///
/// Given each new Classic circuit and its Classic car
/// And every surface a car can reach on it: road, run-off and zones, up to the walls
/// When the car accelerates from a stop on each of those surfaces for 5 s
/// Then it reaches 2 m/s and is pulling away (a car stuck on one stays there for the rest of the race, a player too)
#[test]
fn test_every_car_can_drive_off_every_surface_of_its_circuits() {
    use tdrace_core::physics::car::{Car, CarControls};
    let mut failures = Vec::new();
    for (id, car) in CIRCUITS {
        let track = tdrace_core::catalog::official_track("classic", id);
        let mut surfaces = std::collections::HashSet::new();
        for s in track.spline.samples.iter().step_by(4) {
            // The drivable band, wall to wall (3 m of run-off where a side has no wall).
            let left = s.width * 0.5 + if s.left_wall { s.left_wall_distance.unwrap_or(0.0) } else { 3.0 };
            let right = s.width * 0.5 + if s.right_wall { s.right_wall_distance.unwrap_or(0.0) } else { 3.0 };
            let mut lateral = -right + 0.5;
            while lateral <= left - 0.5 {
                surfaces.insert(track.sample_surface(s.point + s.normal * lateral));
                lateral += 1.0;
            }
        }
        for surface in surfaces {
            let mut c = Car::new(car());
            for _ in 0..600 {
                c.step_per_wheel(&CarControls { throttle: 1.0, ..Default::default() }, [surface; 4], 1.0 / 120.0);
            }
            if c.state.speed < 2.0 {
                failures.push(format!("{id}: {surface:?}: {:.1} m/s after 5 s", c.state.speed));
            }
        }
    }
    assert!(failures.is_empty(), "{:#?}", failures);
}

/// Scenario: a car turns the way it steers on every surface of its circuits (tdrace-le75)
///
/// Given each new Classic circuit and its Classic car
/// And every surface a car can reach on it
/// When the car rolls forward, or backwards, and steers to the left (to the right when it reverses) for 2 s
/// Then it turns to the left
/// (rolling drag pushed along a steered wheel: in deep snow and mud the off-road car turned right when it steered
/// left, and bots on at_frostbite_pass and at_mudbath_valley could not turn round)
#[test]
fn test_every_car_turns_the_way_it_steers_on_every_surface_of_its_circuits() {
    use tdrace_core::physics::car::{Car, CarControls};
    let mut failures = Vec::new();
    for (id, car) in CIRCUITS {
        let track = tdrace_core::catalog::official_track("classic", id);
        let mut surfaces = std::collections::HashSet::new();
        for s in track.spline.samples.iter().step_by(4) {
            let left = s.width * 0.5 + if s.left_wall { s.left_wall_distance.unwrap_or(0.0) } else { 3.0 };
            let right = s.width * 0.5 + if s.right_wall { s.right_wall_distance.unwrap_or(0.0) } else { 3.0 };
            let mut lateral = -right + 0.5;
            while lateral <= left - 0.5 {
                surfaces.insert(track.sample_surface(s.point + s.normal * lateral));
                lateral += 1.0;
            }
        }
        for surface in surfaces {
            for reverse in [false, true] {
                let mut c = Car::new(car());
                let mut controls = CarControls { throttle: 0.6, reverse, ..Default::default() };
                for _ in 0..120 {
                    c.step_per_wheel(&controls, [surface; 4], 1.0 / 120.0);
                }
                // Negative steer turns to the left; reversing, the opposite lock does.
                controls.steer = if reverse { 1.0 } else { -1.0 };
                // Summed from the yaw rate: a kart turns more than half a circle, and the angle wraps round.
                let mut turned = 0.0;
                for _ in 0..240 {
                    c.step_per_wheel(&controls, [surface; 4], 1.0 / 120.0);
                    turned += c.state.angular_velocity / 120.0;
                }
                if turned < 0.2 {
                    failures.push(format!("{id}: {surface:?}, reverse {reverse}: turned {turned:.2} rad in 2 s"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{:#?}", failures);
}

/// Scenario: a bot with its nose on a wall backs off it (tdrace-sw708)
///
/// Given the Classic rally car on rx_hilltop_leap just after it slid sideways into the joker wall at the merge,
/// with its nose on the wall at 1.8 m/s (the pose of a seeded harness race)
/// When its bot drives on for 1.5 s
/// Then the bot reverses (before the fix it ground along the wall at full lock for 3 s, until the no-progress
/// watchdog reversed it, and the car went 10-11 s without progress)
#[test]
fn test_a_bot_with_its_nose_on_a_wall_reverses_off_it() {
    use race_kit::{RaceFormat, RaceRules, RaceWorld};
    use tdrace_core::physics::car::Car;
    use tdrace_core::track::checkpoint::TrackProgressTracker;
    let track = tdrace_core::catalog::official_track("classic", "rx_hilltop_leap");
    let mut car = Car::new(ClassicGameModule::car_classic_rally()).with_pose(glam::Vec2::new(-103.83, -0.12), 1.512);
    car.state.velocity = glam::Vec2::new(1.58, 0.79);
    car.state.speed = car.state.velocity.length();
    car.state.angular_velocity = 0.71;
    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules { format: RaceFormat::TimeAttack, ..RaceRules::default() });
    world.spawn(car, TrackProgressTracker::new(track.checkpoints.len(), 3));
    let mut bot = sample_bot(DrivingStyle::Smooth, DriverTier::Rookie, 261);
    bot.total_distance_travelled = 100.0; // racing, past the grid run-up
    let dt = 1.0 / 120.0;
    let mut reversed = false;
    for _ in 0..(1.5 / dt) as usize {
        let c = bot.compute_controls(&world.vehicles[0], &track, &[], dt);
        reversed |= c.reverse;
        world.step(&track, &[c], dt);
    }
    assert!(reversed, "the bot kept driving into the wall for 1.5 s");
}

/// Scenario: a car under a bridge is hidden, its markers are not
///
/// Given Hangar Sprint and the point where its bridge crosses the lower road
/// When a car is there on the lower road, or on the deck
/// Then only the car on the lower road counts as under the deck (the game draws its markers above the deck)
#[test]
fn test_a_car_on_the_lower_road_is_under_the_bridge_deck() {
    use tdrace_app::game::is_under_bridge_deck;
    let track = tdrace_core::catalog::official_track("vault", "kart_hangar_sprint");
    let s = &track.spline.samples;
    let deck = s.iter().filter(|b| b.is_bridge && b.elevation > 4.0).collect::<Vec<_>>();
    let (lower, top) = s
        .iter()
        .filter(|l| !l.is_bridge && l.elevation < 0.1)
        .flat_map(|l| deck.iter().map(move |b| (l, *b)))
        .min_by(|a, b| a.0.point.distance(a.1.point).total_cmp(&b.0.point.distance(b.1.point)))
        .expect("Hangar Sprint has a crossing");
    assert!(lower.point.distance(top.point) < 1.0, "no crossing point found");
    assert!(is_under_bridge_deck(&track, lower.point, lower.elevation));
    assert!(!is_under_bridge_deck(&track, top.point, top.elevation));
    let open_road = s.iter().find(|l| l.elevation < 0.1 && !is_under_bridge_deck(&track, l.point, 0.0));
    assert!(open_road.is_some());
}
