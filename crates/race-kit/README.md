# race-kit

A headless race world for top-down racing games. It runs the race step, counts laps, gives each
vehicle a finish position and a real finish time, handles wrecks and DNF, and reports what
happened as events. It has no window, sound, database or clock, and builds for
`wasm32-unknown-unknown`. Spec: [`specs/056_racekit_headless_race_world.md`](../../specs/056_racekit_headless_race_world.md).

## Use it from another repo

```toml
[dependencies]
race-kit = { git = "https://github.com/magomar/tdrace", tag = "platform-v0.1.0" }
```

Cargo also fetches the repo's `tracks` submodule, which is the private `tdrace-tracks` repo. The
build needs read access to it (an SSH key for `git@github.com:magomar/tdrace-tracks.git`). The
shared crates do not use it.

## The race loop

```rust
use arcade_race_core::track::{create_prototypical_track, RaceDirection, TrackProgressTracker, TrackShape};
use race_kit::ai::{BotAiDriver, BotProfile};
use race_kit::{RaceEvent, RaceFormat, RaceRules, RaceWorld};
use wheelbase::{Car, CarConfig};

let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules { format: RaceFormat::Laps(3), ..RaceRules::default() });
let slot = track.grid_positions[0];
world.spawn(Car::new(CarConfig::sports_car()).with_pose(slot.position, slot.angle),
            TrackProgressTracker::new(track.checkpoints.len(), 3));
let mut bot = BotAiDriver::with_seed(BotProfile::pro(), 1);

let dt = 1.0 / 120.0;
while !world.is_finished(0) {
    let controls = [bot.compute_controls(&world.vehicles[0], &track, &[], dt)];
    for event in world.step(&track, &controls, dt) {
        if let RaceEvent::Finished { time, .. } = event {
            println!("finished in {time:.2} s");
        }
    }
}
let results = world.results(&track);
```

- `RaceWorld::step` runs surfaces, slipstream, the vehicle step, tree canopy, ramps, vehicle and
  wall collisions, and lap tracking, always in this order.
- Play sounds and effects from the returned `RaceEvent`s, in the order they come.
- Any vehicle model can race: implement `race_kit::Vehicle` (and `arcade_race_core::Body2D`).
  `on_impact` returning `Some` wrecks it.
- `world.results(&track)` gives real times. A vehicle still racing gets a projected time, marked
  `projected`.

For drawing, see [`race-ui`](../race-ui/README.md).
